use crate::api::jwt::RepoTokenPayload;
use crate::api::locks::body::{
    CreateLockPayload, DeleteLockPayload, ListLocksForVerificationPayload, ListLocksQuery,
};
use crate::api::locks::response::{
    CreateLockResponse, DeleteLockResponse, ListLocksForVerificationResponse, ListLocksResponse,
    Lock,
};
use crate::api::repo_query::QueryRepo;
use crate::services::jwt::Jwt;
use crate::traits;
use crate::traits::locks::{LocksProvider, LocksProviderError};
use crate::traits::services::Services;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
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

    // 2) List the locks
    locks_provider
        .list_locks(
            repo,
            path.filter(|s| !s.is_empty()),
            id.filter(|s| !s.is_empty()),
            cursor.filter(|s| !s.is_empty()),
            limit.map(|q| q.parse::<u64>().unwrap_or(100)),
            None,
        )
        .await
        .map(|(next_cursor, locks)| (user, next_cursor, locks))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
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
