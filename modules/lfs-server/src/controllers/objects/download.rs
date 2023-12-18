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
    let is_link_ok = services
        .file_storage_link_signer()
        .check_link(
            &query.repo,
            oid.as_str(),
            Some(&headers),
            Operation::Download,
        )
        .await;
    if !is_link_ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            String::from("Link verification failed"),
        ));
    }

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
                String::from("Download error"),
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

#[cfg(test)]
mod tests {
    use super::download_object;
    use crate::{
        api::repo_query::QueryRepo,
        services::injected_services::InjectedServices,
        test_utils::mocks::{get_mock, MockConfig},
    };
    use axum::{
        extract::{Path, Query, State},
        http::{header, HeaderMap, StatusCode},
    };
    use std::sync::Arc;

    fn download(
        headers: HeaderMap,
        repo: &str,
        services: InjectedServices,
        path: &str,
    ) -> Result<(HeaderMap, Vec<u8>), (StatusCode, String)> {
        crate::aw!(download_object(
            headers,
            Query(QueryRepo::new(String::from(repo))),
            State(Arc::new(services)),
            Path(String::from(path)),
        ))
    }

    #[test]
    fn test_download_object_bad_signature() {
        let services = get_mock(MockConfig {
            check_link_succeed: false,
            ..MockConfig::default()
        });

        let (status_code, message) =
            download(HeaderMap::new(), "a/b/c", services, "/").unwrap_err();

        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Link verification failed");
    }

    #[test]
    fn test_download_object_missing_proxy() {
        let services = get_mock(MockConfig {
            ..MockConfig::default()
        });

        let (status_code, message) =
            download(HeaderMap::new(), "a/b/c", services, "/").unwrap_err();

        assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "No proxy implementation");
    }

    #[test]
    fn test_download_download_error() {
        let services = get_mock(MockConfig {
            proxy_enabled: true,
            proxy_get_success: false,
            ..MockConfig::default()
        });

        let (status_code, message) =
            download(HeaderMap::new(), "a/b/c", services, "/").unwrap_err();

        assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "Download error");
    }

    #[test]
    fn test_download_ok() {
        let services = get_mock(MockConfig {
            proxy_enabled: true,
            ..MockConfig::default()
        });

        let (headers, bytes) = download(HeaderMap::new(), "a/b/c", services, "/").unwrap();

        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
        assert_eq!(bytes, vec![1, 2, 3]);
    }
}
