CREATE TABLE IF NOT EXISTS app.request_participants (
  request_id UUID NOT NULL REFERENCES app.requests(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  source VARCHAR(20) NOT NULL CHECK (source IN ('owner', 'assignee', 'actor')),
  first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (request_id, user_id),
  CHECK (last_seen_at >= first_seen_at)
);

CREATE INDEX IF NOT EXISTS idx_request_participants_user_request
ON app.request_participants(user_id, request_id);

CREATE INDEX IF NOT EXISTS idx_request_participants_request_last_seen
ON app.request_participants(request_id, last_seen_at DESC);

CREATE OR REPLACE FUNCTION app.upsert_request_participant(
  participant_request_id UUID,
  participant_user_id UUID,
  participant_source TEXT,
  participant_seen_at TIMESTAMPTZ
)
RETURNS VOID AS $$
BEGIN
  IF participant_request_id IS NULL OR participant_user_id IS NULL THEN
    RETURN;
  END IF;

  INSERT INTO app.request_participants (
    request_id,
    user_id,
    source,
    first_seen_at,
    last_seen_at
  )
  VALUES (
    participant_request_id,
    participant_user_id,
    participant_source,
    participant_seen_at,
    participant_seen_at
  )
  ON CONFLICT (request_id, user_id)
  DO UPDATE SET
    last_seen_at = GREATEST(
      app.request_participants.last_seen_at,
      EXCLUDED.last_seen_at
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION app.sync_request_participants_from_requests()
RETURNS TRIGGER AS $$
BEGIN
  PERFORM app.upsert_request_participant(
    NEW.id,
    NEW.owner_user_id,
    'owner',
    COALESCE(NEW.updated_at, NEW.created_at, NOW())
  );
  PERFORM app.upsert_request_participant(
    NEW.id,
    NEW.assignee_user_id,
    'assignee',
    COALESCE(NEW.updated_at, NEW.created_at, NOW())
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION app.sync_request_participants_from_audit_logs()
RETURNS TRIGGER AS $$
DECLARE
  old_assignee_email TEXT;
  new_assignee_email TEXT;
  old_assignee_user_id UUID;
  new_assignee_user_id UUID;
BEGIN
  PERFORM app.upsert_request_participant(
    NEW.request_id,
    NEW.actor_user_id,
    'actor',
    NEW.created_at
  );

  old_assignee_email := NULLIF(NEW.old_value ->> 'assignee_email', '');
  new_assignee_email := NULLIF(NEW.new_value ->> 'assignee_email', '');

  IF old_assignee_email IS NOT NULL THEN
    SELECT u.id
    INTO old_assignee_user_id
    FROM auth.users u
    WHERE lower(u.email::text) = lower(old_assignee_email)
    LIMIT 1;

    PERFORM app.upsert_request_participant(
      NEW.request_id,
      old_assignee_user_id,
      'assignee',
      NEW.created_at
    );
  END IF;

  IF new_assignee_email IS NOT NULL THEN
    SELECT u.id
    INTO new_assignee_user_id
    FROM auth.users u
    WHERE lower(u.email::text) = lower(new_assignee_email)
    LIMIT 1;

    PERFORM app.upsert_request_participant(
      NEW.request_id,
      new_assignee_user_id,
      'assignee',
      NEW.created_at
    );
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS requests_sync_participants ON app.requests;
CREATE TRIGGER requests_sync_participants
AFTER INSERT OR UPDATE OF owner_user_id, assignee_user_id ON app.requests
FOR EACH ROW
EXECUTE FUNCTION app.sync_request_participants_from_requests();

DROP TRIGGER IF EXISTS request_audit_logs_sync_participants ON app.request_audit_logs;
CREATE TRIGGER request_audit_logs_sync_participants
AFTER INSERT ON app.request_audit_logs
FOR EACH ROW
EXECUTE FUNCTION app.sync_request_participants_from_audit_logs();

INSERT INTO app.request_participants (
  request_id,
  user_id,
  source,
  first_seen_at,
  last_seen_at
)
SELECT
  req.id,
  req.owner_user_id,
  'owner',
  req.created_at,
  COALESCE(req.updated_at, req.created_at)
FROM app.requests req
ON CONFLICT (request_id, user_id)
DO UPDATE SET
  last_seen_at = GREATEST(
    app.request_participants.last_seen_at,
    EXCLUDED.last_seen_at
  );

INSERT INTO app.request_participants (
  request_id,
  user_id,
  source,
  first_seen_at,
  last_seen_at
)
SELECT
  req.id,
  req.assignee_user_id,
  'assignee',
  req.created_at,
  COALESCE(req.updated_at, req.created_at)
FROM app.requests req
WHERE req.assignee_user_id IS NOT NULL
ON CONFLICT (request_id, user_id)
DO UPDATE SET
  last_seen_at = GREATEST(
    app.request_participants.last_seen_at,
    EXCLUDED.last_seen_at
  );

INSERT INTO app.request_participants (
  request_id,
  user_id,
  source,
  first_seen_at,
  last_seen_at
)
SELECT
  actor_events.request_id,
  actor_events.user_id,
  'actor',
  actor_events.first_seen_at,
  actor_events.last_seen_at
FROM (
  SELECT
    logs.request_id,
    logs.actor_user_id AS user_id,
    MIN(logs.created_at) AS first_seen_at,
    MAX(logs.created_at) AS last_seen_at
  FROM app.request_audit_logs logs
  GROUP BY logs.request_id, logs.actor_user_id
) AS actor_events
ON CONFLICT (request_id, user_id)
DO UPDATE SET
  last_seen_at = GREATEST(
    app.request_participants.last_seen_at,
    EXCLUDED.last_seen_at
  );

INSERT INTO app.request_participants (
  request_id,
  user_id,
  source,
  first_seen_at,
  last_seen_at
)
SELECT
  assignee_events.request_id,
  assignee_events.user_id,
  'assignee',
  assignee_events.first_seen_at,
  assignee_events.last_seen_at
FROM (
  SELECT
    logs.request_id,
    users.id AS user_id,
    MIN(logs.created_at) AS first_seen_at,
    MAX(logs.created_at) AS last_seen_at
  FROM app.request_audit_logs logs
  CROSS JOIN LATERAL (
    VALUES
      (NULLIF(logs.old_value ->> 'assignee_email', '')),
      (NULLIF(logs.new_value ->> 'assignee_email', ''))
  ) AS emails(email)
  JOIN auth.users users
    ON lower(users.email::text) = lower(emails.email)
  GROUP BY logs.request_id, users.id
) AS assignee_events
ON CONFLICT (request_id, user_id)
DO UPDATE SET
  last_seen_at = GREATEST(
    app.request_participants.last_seen_at,
    EXCLUDED.last_seen_at
  );

ALTER TABLE app.request_participants ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS request_participants_select_own ON app.request_participants;
CREATE POLICY request_participants_select_own ON app.request_participants
FOR SELECT
USING (auth.uid() = user_id);

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_publication
    WHERE pubname = 'supabase_realtime'
  ) THEN
    IF NOT EXISTS (
      SELECT 1
      FROM pg_publication_tables
      WHERE pubname = 'supabase_realtime'
        AND schemaname = 'app'
        AND tablename = 'request_participants'
    ) THEN
      ALTER PUBLICATION supabase_realtime ADD TABLE app.request_participants;
    END IF;
  END IF;
END;
$$;
