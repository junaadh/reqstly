use axum::{Json, http::StatusCode};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Serialize)]
pub struct ApiListResponse<T>
where
    T: Serialize,
{
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ResponseMeta {
    pub request_id: String,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub request_id: String,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

pub fn ok<T>(status: StatusCode, data: T) -> (StatusCode, Json<ApiResponse<T>>)
where
    T: Serialize,
{
    (
        status,
        Json(ApiResponse {
            data,
            meta: ResponseMeta {
                request_id: Uuid::new_v4().to_string(),
            },
        }),
    )
}

pub fn list<T>(
    items: Vec<T>,
    page: u64,
    limit: u64,
    total: u64,
) -> (StatusCode, Json<ApiListResponse<T>>)
where
    T: Serialize,
{
    let total_pages = if total == 0 { 0 } else { total.div_ceil(limit) };

    (
        StatusCode::OK,
        Json(ApiListResponse {
            data: items,
            meta: ListMeta {
                request_id: Uuid::new_v4().to_string(),
                total,
                page,
                limit,
                total_pages,
            },
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_sets_total_pages_with_div_ceil() {
        let (_status, body) = list(vec![1, 2], 1, 2, 5);
        assert_eq!(body.0.meta.total_pages, 3);
    }

    #[test]
    fn list_sets_zero_total_pages_when_empty() {
        let (_status, body) = list(Vec::<i32>::new(), 1, 20, 0);
        assert_eq!(body.0.meta.total_pages, 0);
    }
}
