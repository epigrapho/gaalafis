use crate::{
    api::{
        jwt::RepoTokenPayload,
        locks::{
            body::{
                CreateLockPayload, DeleteLockPayload, ListLocksForVerificationPayload,
                ListLocksQuery,
            },
            response::{
                CreateLockResponse, DeleteLockResponse, ListLocksForVerificationResponse,
                ListLocksResponse, Lock,
            },
        },
        repo_query::QueryRepo,
    },
    services::jwt::Jwt,
    traits::{
        self,
        locks::{LocksProvider, LocksProviderError},
        services::Services,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;

fn verify_lock_jwt(
    repo: &str,
    headers: HeaderMap,
    services: &State<Arc<dyn Services + Send + Sync + 'static>>,
    requires_write_access: bool,
) -> Result<String, (StatusCode, String)> {
    // 1) Verify that user has write access to files
    let token_decoder = services.token_encoder_decoder();
    let jwt = Jwt::from_headers(&headers, token_decoder)?;
    let jwt_payload = RepoTokenPayload::new(&jwt)?;
    if requires_write_access && !jwt_payload.has_write_access() {
        return Err((StatusCode::UNAUTHORIZED, String::from("Unauthorized")));
    }

    // 2) Get and verify the repo
    if !jwt_payload.has_access(repo) {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    Ok(jwt_payload.user)
}

fn get_locks_provider<'a>(
    services: &'a State<Arc<dyn Services + Send + Sync + 'static>>,
) -> Result<&'a dyn LocksProvider, (StatusCode, String)> {
    services.locks_provider().ok_or((
        StatusCode::NOT_IMPLEMENTED,
        String::from("The lock api is not implemented on this server"),
    ))
}

async fn list_locks_helper(
    headers: HeaderMap,
    services: &State<Arc<dyn Services + Send + Sync + 'static>>,
    repo: &str,
    path: Option<&str>,
    id: Option<&str>,
    (limit, cursor): (Option<&str>, Option<&str>),
    _ref_name: Option<&str>, // Specification prohibit the use of ref name to filter
) -> Result<(String, Option<String>, Vec<traits::locks::Lock>), (StatusCode, String)> {
    // 1) Preparation
    let user = verify_lock_jwt(repo, headers, services, false)?;
    let locks_provider = get_locks_provider(services)?;

    // 2) Handle limit
    let safe_limit = match limit.map(|q| q.parse::<u64>()) {
        None => None,
        Some(Err(_)) => return Err((StatusCode::BAD_REQUEST, "InvalidLimit".to_string())),
        Some(Ok(limit)) => Some(limit),
    };

    // 2) List the locks
    locks_provider
        .list_locks(
            repo,
            path.filter(|s| !s.is_empty()),
            id.filter(|s| !s.is_empty()),
            cursor.filter(|s| !s.is_empty()),
            safe_limit,
            None,
        )
        .await
        .map(|(next_cursor, locks)| (user, next_cursor, locks))
        .map_err(|err| match err {
            LocksProviderError::InvalidId => (StatusCode::BAD_REQUEST, err.to_string()),
            LocksProviderError::InvalidCursor => (StatusCode::BAD_REQUEST, err.to_string()),
            LocksProviderError::InvalidLimit => (StatusCode::BAD_REQUEST, err.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        })
}

pub async fn post_lock(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    services: State<Arc<dyn Services + Send + Sync + 'static>>,
    Json(payload): Json<CreateLockPayload>,
) -> Result<(StatusCode, Json<CreateLockResponse>), (StatusCode, String)> {
    // 1) Preparation
    let repo = &query.repo;
    let user = verify_lock_jwt(repo, headers, &services, true)?;
    let locks_provider = get_locks_provider(&services)?;

    // 2) Create the lock
    let ref_name = payload.ref_.as_ref().map(|r| r.name.as_str());
    let (lock, new) = locks_provider
        .create_lock(repo, &user, &payload.path, ref_name)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    // 3) In any case, we will return the lock
    let lock = Lock::new(lock.id, lock.path, lock.locked_at, lock.owner.name);

    if new {
        Ok((StatusCode::CREATED, Json(CreateLockResponse::new(lock))))
    } else {
        Ok((
            StatusCode::CONFLICT,
            Json(CreateLockResponse::with_message(
                lock,
                String::from("already created lock"),
            )),
        ))
    }
}

pub async fn list_locks(
    headers: HeaderMap,
    query: Query<ListLocksQuery>,
    services: State<Arc<dyn Services + Send + Sync + 'static>>,
) -> Result<Json<ListLocksResponse>, (StatusCode, String)> {
    let (_, next_cursor, locks) = list_locks_helper(
        headers,
        &services,
        &query.repo,
        query.path.as_deref(),
        query.id.as_deref(),
        (query.limit.as_deref(), query.cursor.as_deref()),
        query.refspec.as_deref(),
    )
    .await?;

    let response_locks: Vec<Lock> = locks
        .into_iter()
        .map(|lock| Lock::new(lock.id, lock.path, lock.locked_at, lock.owner.name))
        .collect();

    Ok(Json(ListLocksResponse::new(response_locks, next_cursor)))
}

pub async fn list_locks_for_verification(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    services: State<Arc<dyn Services + Send + Sync + 'static>>,
    Json(payload): Json<ListLocksForVerificationPayload>,
) -> Result<Json<ListLocksForVerificationResponse>, (StatusCode, String)> {
    // 1) List from backend
    let (user, next_cursor, locks) = list_locks_helper(
        headers,
        &services,
        &query.repo,
        None,
        None,
        (payload.limit.as_deref(), payload.cursor.as_deref()),
        payload.ref_.map(|r| r.name).as_deref(),
    )
    .await?;

    // 2) Separate locks between ours and theirs
    let (ours, theirs) = locks
        .into_iter()
        .map(|lock| Lock::new(lock.id, lock.path, lock.locked_at, lock.owner.name))
        .partition(|l| l.is_owner(&user));

    // 3) Return the locks
    Ok(Json(ListLocksForVerificationResponse::new(
        ours,
        theirs,
        next_cursor,
    )))
}

pub async fn unlock(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    services: State<Arc<dyn Services + Send + Sync + 'static>>,
    Path(id): Path<String>,
    Json(payload): Json<DeleteLockPayload>,
) -> Result<Json<DeleteLockResponse>, (StatusCode, String)> {
    // 1) Preparation
    let repo = &query.repo;
    let user = verify_lock_jwt(repo, headers, &services, true)?;
    let locks_provider = get_locks_provider(&services)?;

    // 2) Delete the locks
    //    Rq. As noted by the specification, the refspec arg can't be used to filter the lock to delete
    //      It shall only be used for authentication purposes:
    //      "Locking API implementations should also only use it for authentication, until advanced locking scenarios have been developed"
    let force = payload.force;
    let lock = locks_provider
        .delete_lock(repo, &user, &id, None, force)
        .await
        .map_err(|err| match err {
            LocksProviderError::LockNotFound => (StatusCode::NOT_FOUND, err.to_string()),
            LocksProviderError::ForceDeleteRequired => (StatusCode::FORBIDDEN, err.to_string()),
            LocksProviderError::InvalidId => (StatusCode::BAD_REQUEST, err.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        })?;

    // 3) Return the deleted lock
    let lock = Lock::new(lock.id, lock.path, lock.locked_at, lock.owner.name);
    Ok(Json(DeleteLockResponse::new(lock)))
}

#[cfg(test)]
mod tests {
    use super::{list_locks, list_locks_for_verification, post_lock, unlock, verify_lock_jwt};
    use crate::{
        api::{
            enums::Operation,
            locks::body::{
                CreateLockPayload, DeleteLockPayload, ListLocksForVerificationPayload,
                ListLocksQuery,
            },
            repo_query::QueryRepo,
        },
        services::injected_services::InjectedServices,
        test_utils::{
            helpers::{assert_http_error, test_auth_headers},
            mocks::{get_mock, DecodedTokenMock, MockConfig},
        },
    };
    use axum::{
        extract::{Json, Path, Query, State},
        http::{HeaderMap, StatusCode},
    };
    use std::sync::Arc;

    /**
     * Test that every controller handle errors from the verify_lock_jwt function
     * In this module we focus on the "Authorization header not found"
     */
    mod test_missing_header {
        use super::*;

        type FnToBeTested<T> = Box<
            dyn Fn(&str, HeaderMap, Arc<InjectedServices>, bool) -> Result<T, (StatusCode, String)>,
        >;

        fn test_missing_header<T>(to_be_tested: FnToBeTested<T>) {
            let services = get_mock(MockConfig::default());
            assert_http_error(
                to_be_tested("a/b/c", HeaderMap::new(), Arc::new(services), false),
                StatusCode::UNAUTHORIZED,
                "Authorization header not found",
            );
        }

        #[test]
        fn test_missing_header_verify_lock_jwt() {
            test_missing_header(Box::new(
                |repo, headers, services, requires_write_access| {
                    verify_lock_jwt(repo, headers, &State(services), requires_write_access)
                },
            ));
        }

        #[test]
        fn test_missing_header_post_lock() {
            test_missing_header(Box::new(|repo, headers, services, _| {
                crate::aw!(post_lock(
                    headers,
                    Query(QueryRepo::new(repo.to_string())),
                    State(services),
                    Json(CreateLockPayload::new("path", Some("ref"))),
                ))
            }));
        }

        #[test]
        fn test_missing_header_list_locks() {
            test_missing_header(Box::new(|repo, headers, services, _| {
                crate::aw!(list_locks(
                    headers,
                    Query(ListLocksQuery {
                        repo: repo.to_string(),
                        ..ListLocksQuery::default()
                    }),
                    State(services),
                ))
            }));
        }

        #[test]
        fn test_missing_header_list_locks_for_verification() {
            test_missing_header(Box::new(|repo, headers, services, _| {
                crate::aw!(list_locks_for_verification(
                    headers,
                    Query(QueryRepo::new(repo.to_string())),
                    State(services),
                    Json(ListLocksForVerificationPayload::default())
                ))
            }));
        }

        #[test]
        fn test_missing_header_unlock() {
            test_missing_header(Box::new(|repo, headers, services, _| {
                crate::aw!(unlock(
                    headers,
                    Query(QueryRepo::new(repo.to_string())),
                    State(services),
                    Path("id".to_string()),
                    Json(DeleteLockPayload::default())
                ))
            }));
        }
    }

    /**
     * Test that every type of bad token is handled by verify_lock_jwt
     */
    mod test_verify_lock_jwt {
        use super::*;

        #[test]
        fn test_verify_loc_jwt_bad_header() {
            let services = get_mock(MockConfig::default());
            assert_http_error(
                verify_lock_jwt(
                    "a/b/c",
                    test_auth_headers("token"),
                    &State(Arc::new(services)),
                    false,
                ),
                StatusCode::UNAUTHORIZED,
                "Failed to parse Authorization header",
            );
        }

        #[test]
        fn test_verify_loc_jwt_token_expired() {
            let services = get_mock(MockConfig {
                expired: true,
                ..MockConfig::default()
            });
            assert_http_error(
                verify_lock_jwt(
                    "a/b/c",
                    test_auth_headers("Bearer token"),
                    &State(Arc::new(services)),
                    false,
                ),
                StatusCode::UNAUTHORIZED,
                "Token expired",
            );
        }

        #[test]
        fn test_verify_loc_jwt_token_missing_write_authorization() {
            let services = get_mock(MockConfig {
                ..MockConfig::default()
            });

            assert_http_error(
                verify_lock_jwt(
                    "a/b/c",
                    test_auth_headers("Bearer token"),
                    &State(Arc::new(services)),
                    true,
                ),
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
            );
        }

        #[test]
        fn test_verify_loc_jwt_token_wrong_repo() {
            let services = get_mock(MockConfig {
                ..MockConfig::default()
            });

            assert_http_error(
                verify_lock_jwt(
                    "another",
                    test_auth_headers("Bearer token"),
                    &State(Arc::new(services)),
                    false,
                ),
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
            );
        }
    }

    /**
     * Test that every controller handle correctly missing locks provider
     */
    mod test_missing_locks_provider {
        use super::*;

        fn get_prepared_download_services() -> Arc<InjectedServices> {
            Arc::new(get_mock(MockConfig::default()))
        }
        fn get_prepared_upload_services() -> Arc<InjectedServices> {
            Arc::new(get_mock(MockConfig {
                decoded: Some(DecodedTokenMock {
                    operation: Operation::Upload,
                    repo: String::from("a/b/c"),
                }),
                ..MockConfig::default()
            }))
        }

        fn expect_not_implemented<T>(res: Result<T, (StatusCode, String)>) {
            assert_http_error(
                res,
                StatusCode::NOT_IMPLEMENTED,
                "The lock api is not implemented on this server",
            );
        }

        #[test]
        fn test_missing_locks_provider_post_lock() {
            expect_not_implemented(crate::aw!(post_lock(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new("a/b/c".to_string())),
                State(get_prepared_upload_services()),
                Json(CreateLockPayload::new("path", Some("ref"))),
            )));
        }

        #[test]
        fn test_missing_locks_provider_list_locks() {
            expect_not_implemented(crate::aw!(list_locks(
                test_auth_headers("Bearer token"),
                Query(ListLocksQuery {
                    repo: "a/b/c".to_string(),
                    ..ListLocksQuery::default()
                }),
                State(get_prepared_download_services()),
            )));
        }

        #[test]
        fn test_missing_locks_provider_list_locks_for_verification() {
            expect_not_implemented(crate::aw!(list_locks_for_verification(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new("a/b/c".to_string())),
                State(get_prepared_download_services()),
                Json(ListLocksForVerificationPayload::default())
            )));
        }

        #[test]
        fn test_missing_locks_provider_unlock() {
            expect_not_implemented(crate::aw!(unlock(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new("a/b/c".to_string())),
                State(get_prepared_upload_services()),
                Path("id".to_string()),
                Json(DeleteLockPayload::default())
            )));
        }
    }

    /**
     * Test the post_lock controller
     */
    mod test_post_lock {
        use super::*;
        use crate::api::locks::response::CreateLockResponse;

        fn run_post_lock(path: &str) -> (StatusCode, Json<CreateLockResponse>) {
            let services = get_mock(MockConfig {
                decoded: Some(DecodedTokenMock {
                    operation: Operation::Upload,
                    repo: String::from("a/b/c"),
                }),
                locks_enabled: true,
                ..MockConfig::default()
            });
            crate::aw!(post_lock(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new("a/b/c".to_string())),
                State(Arc::new(services)),
                Json(CreateLockPayload::new(path, Some("ref"))),
            ))
            .unwrap()
        }

        #[test]
        fn test_post_lock_new() {
            let (status, Json(res)) = run_post_lock("path");
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(
                serde_json::to_string(&res).unwrap(),
                "{\"lock\":{\"id\":\"id\",\"path\":\"path\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}}"
            );
        }

        #[test]
        fn test_post_lock_already_exists() {
            let (status, Json(res)) = run_post_lock("existing");
            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(
                serde_json::to_string(&res).unwrap(),
                "{\"lock\":{\"id\":\"id\",\"path\":\"existing\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},\"message\":\"already created lock\"}"
            );
        }
    }

    /**
     * Test the list_locks controller
     */
    mod test_list_locks {
        use super::*;
        use crate::api::locks::response::ListLocksResponse;

        fn run_list_locks(
            path: Option<&str>,
            id: Option<&str>,
            limit: Option<&str>,
            cursor: Option<&str>,
            refspec: Option<&str>,
        ) -> Result<Json<ListLocksResponse>, (StatusCode, String)> {
            let services = get_mock(MockConfig {
                decoded: Some(DecodedTokenMock {
                    operation: Operation::Download,
                    repo: String::from("a/b/c"),
                }),
                locks_enabled: true,
                ..MockConfig::default()
            });
            crate::aw!(list_locks(
                test_auth_headers("Bearer token"),
                Query(ListLocksQuery {
                    repo: "a/b/c".to_string(),
                    path: path.map(|s| s.to_string()),
                    id: id.map(|s| s.to_string()),
                    limit: limit.map(|s| s.to_string()),
                    cursor: cursor.map(|s| s.to_string()),
                    refspec: refspec.map(|s| s.to_string()),
                }),
                State(Arc::new(services)),
            ))
        }

        #[test]
        fn test_list_locks_invalid_id() {
            assert_http_error(
                run_list_locks(None, Some("invalid-id"), None, None, None),
                StatusCode::BAD_REQUEST,
                "InvalidId",
            );
        }

        #[test]
        fn test_list_locks_invalid_cursor() {
            assert_http_error(
                run_list_locks(None, None, None, Some("invalid-cursor"), None),
                StatusCode::BAD_REQUEST,
                "InvalidCursor",
            );
        }

        #[test]
        fn test_list_locks_unparseable_limit() {
            assert_http_error(
                run_list_locks(None, None, Some("invalid-limit"), None, None),
                StatusCode::BAD_REQUEST,
                "InvalidLimit",
            );
        }

        #[test]
        fn test_list_locks_invalid_limit() {
            assert_http_error(
                run_list_locks(None, None, Some("42"), None, None),
                StatusCode::BAD_REQUEST,
                "InvalidLimit",
            );
        }

        #[test]
        fn test_list_locks_limit0_of_3() {
            let Json(res) = run_list_locks(None, None, Some("0"), None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"locks\":[]}");
        }

        #[test]
        fn test_list_locks_limit2_of_3() {
            let Json(res) = run_list_locks(None, None, Some("2"), None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"locks\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}],\"next_cursor\":\"id3\"}");
        }

        #[test]
        fn test_list_locks_limit4_of_3() {
            let Json(res) = run_list_locks(None, None, Some("4"), None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"locks\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}},{\"id\":\"id4\",\"path\":\"path4\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user4\"}}]}");
        }

        #[test]
        fn test_list_locks_limit10_of_3() {
            let Json(res) = run_list_locks(None, None, Some("10"), None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"locks\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}},{\"id\":\"id4\",\"path\":\"path4\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user4\"}}]}");
        }

        #[test]
        fn test_list_locks() {
            let Json(res) = run_list_locks(None, None, None, None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"locks\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}}],\"next_cursor\":\"id4\"}");
        }
    }

    /**
     * Test the list_locks_for_verification controller
     */
    mod test_list_locks_for_verification {
        use super::*;
        use crate::api::locks::response::ListLocksForVerificationResponse;

        fn run_list_locks_for_verification(
            limit: Option<&str>,
            cursor: Option<&str>,
        ) -> Result<Json<ListLocksForVerificationResponse>, (StatusCode, String)> {
            let services = get_mock(MockConfig {
                locks_enabled: true,
                ..MockConfig::default()
            });
            crate::aw!(list_locks_for_verification(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new(String::from("a/b/c"))),
                State(Arc::new(services)),
                Json(ListLocksForVerificationPayload {
                    limit: limit.map(|s| s.to_string()),
                    cursor: cursor.map(|s| s.to_string()),
                    ref_: None,
                }),
            ))
        }

        #[test]
        fn test_list_locks_invalid_cursor() {
            assert_http_error(
                run_list_locks_for_verification(None, Some("invalid-cursor")),
                StatusCode::BAD_REQUEST,
                "InvalidCursor",
            );
        }

        #[test]
        fn test_list_locks_unparseable_limit() {
            assert_http_error(
                run_list_locks_for_verification(Some("invalid-limit"), None),
                StatusCode::BAD_REQUEST,
                "InvalidLimit",
            );
        }

        #[test]
        fn test_list_locks_invalid_limit() {
            assert_http_error(
                run_list_locks_for_verification(Some("42"), None),
                StatusCode::BAD_REQUEST,
                "InvalidLimit",
            );
        }

        #[test]
        fn test_list_locks_limit0_of_3() {
            let Json(res) = run_list_locks_for_verification(Some("0"), None).unwrap();
            assert_eq!(
                serde_json::to_string(&res).unwrap(),
                "{\"ours\":[],\"theirs\":[]}"
            );
        }

        #[test]
        fn test_list_locks_limit2_of_3() {
            let Json(res) = run_list_locks_for_verification(Some("2"), None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"ours\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}],\"theirs\":[],\"next_cursor\":\"id3\"}");
        }

        #[test]
        fn test_list_locks_limit4_of_3() {
            let Json(res) = run_list_locks_for_verification(Some("4"), None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"ours\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}],\"theirs\":[{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}},{\"id\":\"id4\",\"path\":\"path4\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user4\"}}]}");
        }

        #[test]
        fn test_list_locks_limit10_of_3() {
            let Json(res) = run_list_locks_for_verification(Some("10"), None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"ours\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}],\"theirs\":[{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}},{\"id\":\"id4\",\"path\":\"path4\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user4\"}}]}");
        }

        #[test]
        fn test_list_locks() {
            let Json(res) = run_list_locks_for_verification(None, None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"ours\":[{\"id\":\"id1\",\"path\":\"path1\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}},{\"id\":\"id2\",\"path\":\"path2\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}],\"theirs\":[{\"id\":\"id3\",\"path\":\"path3\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user3\"}}],\"next_cursor\":\"id4\"}");
        }
    }

    /**
     * Test the unlock controller
     */
    mod test_unlock {
        use super::*;
        use crate::api::locks::response::DeleteLockResponse;

        fn run_unlock(
            id: &str,
            force: Option<bool>,
        ) -> Result<Json<DeleteLockResponse>, (StatusCode, String)> {
            let services = get_mock(MockConfig {
                decoded: Some(DecodedTokenMock {
                    operation: Operation::Upload,
                    repo: String::from("a/b/c"),
                }),
                locks_enabled: true,
                ..MockConfig::default()
            });
            crate::aw!(unlock(
                test_auth_headers("Bearer token"),
                Query(QueryRepo::new(String::from("a/b/c"))),
                State(Arc::new(services)),
                Path(id.to_string()),
                Json(DeleteLockPayload { force, ref_: None }),
            ))
        }

        #[test]
        fn test_unlock() {
            let Json(res) = run_unlock("id", None).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"lock\":{\"id\":\"id\",\"path\":\"path\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}}");
        }

        #[test]
        fn test_unlock_invalid_id() {
            assert_http_error(
                run_unlock("invalid-id", Some(false)),
                StatusCode::BAD_REQUEST,
                "InvalidId",
            )
        }

        #[test]
        fn test_unlock_missing_lock() {
            assert_http_error(
                run_unlock("not-found", Some(false)),
                StatusCode::NOT_FOUND,
                "LockNotFound",
            )
        }

        #[test]
        fn test_unlock_someones_else_lock() {
            assert_http_error(
                run_unlock("force-required", None),
                StatusCode::FORBIDDEN,
                "ForceDeleteRequired",
            )
        }

        #[test]
        fn test_unlock_someones_else_lock_explicit_dont_force() {
            assert_http_error(
                run_unlock("force-required", Some(false)),
                StatusCode::FORBIDDEN,
                "ForceDeleteRequired",
            )
        }

        #[test]
        fn test_force_unlock_someones_else_lock() {
            let Json(res) = run_unlock("force-required", Some(true)).unwrap();
            assert_eq!(serde_json::to_string(&res).unwrap(), "{\"lock\":{\"id\":\"force-required\",\"path\":\"path\",\"locked_at\":\"1970-01-01T00:00:00+00:00\",\"owner\":{\"name\":\"user\"}}}");
        }
    }
}
