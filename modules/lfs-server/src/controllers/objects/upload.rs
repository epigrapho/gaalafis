use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};

use crate::{
    api::{enums::Operation, repo_query::QueryRepo},
    traits::services::Services,
};

pub async fn upload_object(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    State(services): State<Arc<dyn Services + Send + Sync + 'static>>,
    Path(oid): Path<String>,
    body: Bytes,
) -> Result<(), (StatusCode, String)> {
    // 1) Extract and validate
    services
        .file_storage_link_signer()
        .check_link(&query.repo, oid.as_str(), Some(&headers), Operation::Upload)
        .await;

    // 2) Get content type
    let content_type = headers
        .get("content-type")
        .map(|v| {
            v.to_str()
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        String::from("Invalid content type"),
                    )
                })
        })
        .unwrap_or(Ok("application/octet-stream"))?;

    // 3) Upload
    services
        .file_storage_proxy()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("No proxy implementation"),
        ))?
        .post(&query.repo, oid.as_str(), body.to_vec(), content_type)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Upload error"),
            )
        })
}
