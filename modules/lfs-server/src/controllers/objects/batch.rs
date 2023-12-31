use axum::{
    extract::{Json, Query, State},
    http::{HeaderMap, StatusCode},
};
use std::sync::Arc;

use crate::{
    api::{
        enums::{HashAlgorithm, Operation, Transfer},
        jwt::RepoTokenPayload,
        objects_batch::{
            body::ObjectsBatchRequestPayload,
            response::{Object, ObjectsBatchSuccessResponse},
        },
        repo_query::QueryRepo,
    },
    services::jwt::Jwt,
    traits::{file_storage::FileStorageMetaResult, services::Services},
};

// Request the ability to transfer a batch of objects
// Might be download or upload
// Implements the Batch api defined at https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md
// Expect a token in the header
// Available at /objects/batch?repo=a/b/c
pub async fn post_objects_batch(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    services: State<Arc<dyn Services + Send + Sync + 'static>>,
    Json(payload): Json<ObjectsBatchRequestPayload>,
) -> Result<Json<ObjectsBatchSuccessResponse>, (StatusCode, String)> {
    // 1) Extract and validate
    let token_decoder = services.token_encoder_decoder();
    let jwt = Jwt::from_headers(&headers, token_decoder)?;
    let jwt_payload = RepoTokenPayload::new(&jwt)?;

    payload.assert_jwt_access_level_higher_than_requested(&jwt_payload)?;

    query.assert_repo_match_token(&jwt_payload)?;

    payload.assert_hash_algo(HashAlgorithm::Sha256)?;

    payload.assert_transfer_accepted(Transfer::Basic)?;

    // 7) For each object, verify if it exists on storage server
    let services = Arc::clone(&services);

    let repo = &query.repo;
    let number_of_objects = payload.objects.len();
    let mut objects: Vec<Object> = Vec::with_capacity(number_of_objects);
    for object in payload.objects.iter() {
        let oid = &object.oid[..];
        let size = object.size;
        let result = services
            .file_storage_meta_requester()
            .get_meta_result(repo, oid)
            .await;

        let FileStorageMetaResult { exists, .. } = result;
        let signer = services.file_storage_link_signer();

        let object = if exists {
            let download = signer.get_presigned_link(result).await;
            match download {
                Ok(download) => Object::download(oid, size, download),
                Err(error) => Object::error(oid, size, error),
            }
        } else if let Operation::Upload = payload.operation {
            let actions = signer.post_presigned_link(result, size).await;
            match actions {
                Ok(actions) => Object::upload(oid, size, actions.0, actions.1),
                Err(error) => Object::error(oid, size, error),
            }
        } else {
            Object::not_found(oid, size)
        };

        objects.push(object);
    }

    // 8) Return the result
    let response = ObjectsBatchSuccessResponse::basic_sha256(objects);

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{
            enums::Operation,
            objects_batch::{
                body::{ObjectIdentity, ObjectsBatchRequestPayload},
                response::ObjectsBatchSuccessResponse,
            },
            repo_query::QueryRepo,
        },
        controllers::objects::batch::post_objects_batch,
        services::injected_services::InjectedServices,
        test_utils::{
            helpers::test_auth_headers,
            mocks::{get_mock, DecodedTokenMock, MockConfig},
        },
    };
    use axum::{
        extract::{Query, State},
        http::{HeaderMap, StatusCode},
        Json,
    };
    use std::sync::Arc;

    fn post(
        headers: HeaderMap,
        repo: &str,
        services: InjectedServices,
        payload: ObjectsBatchRequestPayload,
    ) -> Result<Json<ObjectsBatchSuccessResponse>, (StatusCode, String)> {
        crate::aw!(post_objects_batch(
            headers,
            Query(QueryRepo::new(String::from(repo))),
            State(Arc::new(services)),
            Json(payload),
        ))
    }

    #[test]
    fn test_post_objects_batch_missing_authorization() {
        let services = get_mock(MockConfig {
            ..MockConfig::default()
        });

        let (status_code, message) = post(
            HeaderMap::new(),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![]),
        )
        .unwrap_err();

        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Authorization header not found");
    }

    #[test]
    fn test_post_objects_batch_bad_authorization_token() {
        let services = get_mock(MockConfig {
            ..MockConfig::default()
        });

        let (status_code, message) = post(
            test_auth_headers("token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![]),
        )
        .unwrap_err();

        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Failed to parse Authorization header");
    }

    #[test]
    fn test_post_objects_batch_bad_authorization_token_expired() {
        let services = get_mock(MockConfig {
            expired: true,
            ..MockConfig::default()
        });

        let (status_code, message) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![]),
        )
        .unwrap_err();

        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Token expired");
    }

    #[test]
    fn test_post_objects_batch_no_objects() {
        let services = get_mock(MockConfig {
            ..MockConfig::default()
        });

        let Json(res) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![]),
        )
        .unwrap();

        let json = serde_json::to_string(&res).unwrap();
        assert_eq!(
            json,
            "{\"transfer\":\"basic\",\"objects\":[],\"hash_algo\":\"sha256\"}"
        );
    }

    #[test]
    fn test_post_objects_batch_not_found_object() {
        let services = get_mock(MockConfig {
            found: false,
            ..MockConfig::default()
        });

        let Json(res) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![ObjectIdentity::new(
                "not-found-oid",
                5,
            )]),
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"not-found-oid\",\"size\":5,\"error\":{\"message\":\"Not found\"}}],\"hash_algo\":\"sha256\"}"
        );
    }

    #[test]
    fn test_post_objects_batch_found_object() {
        let services = get_mock(MockConfig {
            size: 50,
            ..MockConfig::default()
        });

        let Json(res) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_download_default(vec![ObjectIdentity::new(
                "found-oid",
                5,
            )]),
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"found-oid\",\"size\":5,\"actions\":{\"download\":{\"href\":\"https://example.com/download/a/b/c/found-oid?size=50\",\"header\":{\"Authorization\":\"token\"},\"expires_in\":60}}}],\"hash_algo\":\"sha256\"}",
        );
    }

    #[test]
    fn test_post_upload_bad_authorization_missing_write_auth() {
        let services = get_mock(MockConfig {
            found: true,
            size: 50,
            ..MockConfig::default()
        });

        let (status_code, message) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_upload_default(vec![ObjectIdentity::new(
                "found-oid",
                5,
            )]),
        )
        .unwrap_err();

        assert_eq!(status_code, StatusCode::FORBIDDEN);
        assert_eq!(message, "You only have read access to this repository");
    }

    #[test]
    fn test_post_upload_objects_batch_found_object_download() {
        let services = get_mock(MockConfig {
            found: true,
            size: 50,
            decoded: Some(DecodedTokenMock {
                operation: Operation::Upload,
                repo: String::from("a/b/c"),
            }),
            ..MockConfig::default()
        });

        let Json(res) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_upload_default(vec![ObjectIdentity::new(
                "found-oid",
                5,
            )]),
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"found-oid\",\"size\":5,\"actions\":{\"download\":{\"href\":\"https://example.com/download/a/b/c/found-oid?size=50\",\"header\":{\"Authorization\":\"token\"},\"expires_in\":60}}}],\"hash_algo\":\"sha256\"}",
        );
    }

    #[test]
    fn test_post_upload_objects_batch_not_found_object() {
        let services = get_mock(MockConfig {
            found: false,
            size: 0,
            with_verify: false,
            decoded: Some(DecodedTokenMock {
                operation: Operation::Upload,
                repo: String::from("a/b/c"),
            }),
            ..MockConfig::default()
        });

        let Json(res) = post(
            test_auth_headers("Bearer token"),
            "a/b/c",
            services,
            ObjectsBatchRequestPayload::new_upload_default(vec![ObjectIdentity::new(
                "found-oid",
                5,
            )]),
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"found-oid\",\"size\":5,\"actions\":{\"upload\":{\"href\":\"https://example.com/upload/a/b/c/found-oid?size=5\",\"header\":{\"Authorization\":\"token\"},\"expires_in\":60}}}],\"hash_algo\":\"sha256\"}",
        );
    }
}
