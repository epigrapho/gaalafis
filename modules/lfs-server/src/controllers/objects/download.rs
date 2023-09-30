use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
};

use crate::{
    api::{enums::Operation, repo_query::QueryRepo},
    traits::services::Services,
};

pub async fn download_object(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    State(services): State<Arc<dyn Services + Send + Sync + 'static>>,
    Path(oid): Path<String>,
) -> Result<(HeaderMap, Vec<u8>), (StatusCode, String)> {
    // 1) Extract and validate
    services
        .file_storage_link_signer()
        .check_link(&query.repo, oid.as_str(), Some(&headers), Operation::Upload)
        .await;

    // 2) Download from proxy
    let (data, content_type) = services
        .file_storage_proxy()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("No proxy implementation"),
        ))?
        .get(&query.repo, &oid)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Upload error"),
            )
        })?;

    // 3) Return, with content type
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CONTENT_TYPE,
        content_type
            .parse()
            .unwrap_or(header::HeaderValue::from_static("application/octet-stream")),
    );
    Ok((response_headers, data))
}
