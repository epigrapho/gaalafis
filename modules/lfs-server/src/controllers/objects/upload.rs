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
    let is_link_ok = services
        .file_storage_link_signer()
        .check_link(&query.repo, oid.as_str(), Some(&headers), Operation::Upload)
        .await;
    if !is_link_ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            String::from("Link verification failed"),
        ));
    }

    // 2) Get content type
    let content_type = headers
        .get("content-type")
        .map(|v| {
            v.to_str().map_err(|_| {
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
        .map_err(|e| {
            tracing::error!("Upload error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Upload error"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::upload_object;
    use crate::{
        api::repo_query::QueryRepo,
        services::injected_services::InjectedServices,
        test_utils::mocks::{get_mock, MockConfig},
    };
    use axum::{
        body::Bytes,
        extract::{Path, Query, State},
        http::{header, HeaderMap, StatusCode},
    };
    use std::sync::Arc;

    fn upload(
        headers: HeaderMap,
        repo: &str,
        services: InjectedServices,
        path: &str,
        body: Vec<u8>,
    ) -> Result<(), (StatusCode, String)> {
        crate::aw!(upload_object(
            headers,
            Query(QueryRepo::new(String::from(repo))),
            State(Arc::new(services)),
            Path(String::from(path)),
            Bytes::from(body),
        ))
    }

    #[test]
    fn test_upload_object_bad_signature() {
        let services = get_mock(MockConfig {
            check_link_succeed: false,
            ..MockConfig::default()
        });

        let (status_code, message) =
            upload(HeaderMap::new(), "a/b/c", services, "/", vec![1, 2, 3]).unwrap_err();

        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Link verification failed");
    }

    #[test]
    fn test_upload_object_missing_proxy() {
        let services = get_mock(MockConfig {
            ..MockConfig::default()
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            "application/octet-stream".parse().unwrap(),
        );

        let (status_code, message) =
            upload(headers, "a/b/c", services, "/", vec![1, 2, 3]).unwrap_err();

        assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "No proxy implementation");
    }

    #[test]
    fn test_upload_upload_error() {
        let services = get_mock(MockConfig {
            proxy_enabled: true,
            proxy_post_success: false,
            ..MockConfig::default()
        });

        let (status_code, message) =
            upload(HeaderMap::new(), "a/b/c", services, "/", vec![1, 2, 3]).unwrap_err();

        assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "Upload error");
    }

    #[test]
    fn test_upload_ok() {
        let services = get_mock(MockConfig {
            proxy_enabled: true,
            ..MockConfig::default()
        });

        upload(HeaderMap::new(), "a/b/c", services, "/", vec![1, 2, 3]).unwrap();
    }
}
