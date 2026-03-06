use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub const WS_QUEUE_DEPTH: usize = 256;
pub const WS_MAX_MESSAGE_BYTES: usize = 16 * 1024;
pub const WS_HEARTBEAT_INTERVAL_SECS: u64 = 25;
pub const WS_IDLE_TIMEOUT_SECS: u64 = 75;

const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope {
    pub v: u8,
    #[serde(rename = "type")]
    pub event_type: String,
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(
        event_type: impl Into<String>,
        request_id: Option<Uuid>,
        trace_id: Option<String>,
        payload: Value,
    ) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            event_type: event_type.into(),
            ts: Utc::now(),
            request_id,
            trace_id,
            payload,
        }
    }

    pub fn encode(self) -> Result<Arc<str>, serde_json::Error> {
        serde_json::to_string(&self).map(Into::<Arc<str>>::into)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello {
        #[serde(default)]
        last_seen_ts: Option<DateTime<Utc>>,
    },
}

impl ClientMessage {
    pub fn last_seen_ts(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Hello { last_seen_ts } => *last_seen_ts,
        }
    }
}

type ConnectionSenders = HashMap<Uuid, mpsc::Sender<Arc<str>>>;

#[derive(Clone, Debug)]
pub struct RealtimeHub {
    inner: Arc<RwLock<HashMap<Uuid, ConnectionSenders>>>,
}

impl RealtimeHub {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(
        &self,
        user_id: Uuid,
    ) -> (Uuid, mpsc::Receiver<Arc<str>>) {
        let connection_id = Uuid::new_v4();
        let (sender, receiver) = mpsc::channel(WS_QUEUE_DEPTH);

        let mut guard = self.inner.write().await;
        guard
            .entry(user_id)
            .or_default()
            .insert(connection_id, sender);

        (connection_id, receiver)
    }

    pub async fn unregister(&self, user_id: Uuid, connection_id: Uuid) {
        let mut guard = self.inner.write().await;

        if let Some(connections) = guard.get_mut(&user_id) {
            connections.remove(&connection_id);
            if connections.is_empty() {
                guard.remove(&user_id);
            }
        }
    }

    pub async fn send_to_user(
        &self,
        user_id: Uuid,
        message: Arc<str>,
    ) -> usize {
        let senders = {
            let guard = self.inner.read().await;
            guard
                .get(&user_id)
                .map(|connections| {
                    connections
                        .iter()
                        .map(|(connection_id, sender)| {
                            (*connection_id, sender.clone())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let mut delivered = 0;
        let mut stale_connection_ids = Vec::new();

        for (connection_id, sender) in senders {
            match sender.try_send(Arc::clone(&message)) {
                Ok(()) => {
                    delivered += 1;
                }
                Err(_) => stale_connection_ids.push(connection_id),
            }
        }

        if !stale_connection_ids.is_empty() {
            let mut guard = self.inner.write().await;
            if let Some(connections) = guard.get_mut(&user_id) {
                for connection_id in stale_connection_ids {
                    connections.remove(&connection_id);
                }

                if connections.is_empty() {
                    guard.remove(&user_id);
                }
            }
        }

        delivered
    }

    pub async fn broadcast_to_users<I>(
        &self,
        user_ids: I,
        message: Arc<str>,
    ) -> usize
    where
        I: IntoIterator<Item = Uuid>,
    {
        let unique_user_ids = user_ids.into_iter().collect::<HashSet<_>>();
        let mut delivered = 0;

        for user_id in unique_user_ids {
            delivered += self.send_to_user(user_id, Arc::clone(&message)).await;
        }

        delivered
    }

    pub async fn connection_count(&self, user_id: Uuid) -> usize {
        let guard = self.inner.read().await;
        guard.get(&user_id).map_or(0, HashMap::len)
    }
}

impl Default for RealtimeHub {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn parse_allowed_origins(raw: &str) -> Vec<String> {
    if raw.trim() == "*" {
        return vec!["*".to_string()];
    }

    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[must_use]
pub fn is_origin_allowed(
    origin: Option<&str>,
    allowed_origins: &[String],
) -> bool {
    if allowed_origins.iter().any(|item| item == "*") {
        return true;
    }

    let Some(origin) = origin else {
        return false;
    };

    allowed_origins.iter().any(|allowed| allowed == origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_allowed_origins_supports_csv_and_star() {
        assert_eq!(parse_allowed_origins("*"), vec!["*"]);
        assert_eq!(
            parse_allowed_origins(
                " https://localhost,https://app.example.com "
            ),
            vec!["https://localhost", "https://app.example.com"]
        );
    }

    #[tokio::test]
    async fn hub_sends_and_cleans_up_connections() {
        let hub = RealtimeHub::new();
        let user_id = Uuid::new_v4();

        let (connection_id, mut receiver) = hub.register(user_id).await;
        let message = EventEnvelope::new(
            "request.created",
            Some(Uuid::new_v4()),
            None,
            json!({"id": Uuid::new_v4()}),
        )
        .encode()
        .expect("event should encode");

        let delivered = hub.send_to_user(user_id, Arc::clone(&message)).await;
        assert_eq!(delivered, 1);

        let received =
            receiver.recv().await.expect("receiver should get event");
        assert_eq!(received, message);

        drop(receiver);

        let delivered_after_drop =
            hub.send_to_user(user_id, Arc::clone(&message)).await;
        assert_eq!(delivered_after_drop, 0);

        hub.unregister(user_id, connection_id).await;
        assert_eq!(hub.connection_count(user_id).await, 0);
    }
}
