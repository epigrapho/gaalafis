use crate::api::jwt::RepoTokenPayload;
use crate::api::locks::body::{CreateLockPayload, DeleteLockPayload, ListLocksQuery};
use crate::api::locks::response::{
    CreateLockResponse, DeleteLockResponse, ListLocksResponse, Lock,
};
use crate::api::repo_query::QueryRepo;
use crate::services::jwt::Jwt;
use crate::traits::locks::{LocksProvider, LocksProviderError};
use crate::traits::services::Services;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

fn discard_if_empty(s: Option<&str>) -> Option<&str> {
    match s {
        Some("") => None,
        s => s,
    }
}

fn verify_lock_jwt(
    repo: &str,
    headers: HeaderMap,
    services: &State<Arc<dyn Services + Send + Sync + 'static>>,
) -> Result<String, (StatusCode, String)> {
    // 1) Verify that user has write access to files
    let token_decoder = services.token_encoder_decoder();
    let jwt = Jwt::from_headers(&headers, token_decoder)?;
    let jwt_payload = RepoTokenPayload::new(&jwt)?;
    if !jwt_payload.has_write_access() {
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
    let user = verify_lock_jwt(repo, headers, &services)?;
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
    // 1) Preparation
    let repo = &query.repo;
    verify_lock_jwt(repo, headers, &services)?;
    let locks_provider = get_locks_provider(&services)?;

    // 2) List the locks
    let (next_cursor, locks) = locks_provider
        .list_locks(
            repo,
            discard_if_empty(query.path.as_deref()),
            discard_if_empty(query.id.as_deref()),
            discard_if_empty(query.cursor.as_deref()),
            query
                .limit
                .as_ref()
                .map(|q| q.parse::<u64>().unwrap_or(100)),
            discard_if_empty(query.refspec.as_deref()),
        )
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    // 3) In any case, we will return the lock
    let locks = locks
        .iter()
        .map(|lock| {
            Lock::new(
                lock.id.clone(),
                lock.path.clone(),
                lock.locked_at,
                lock.owner.name.clone(),
            )
        })
        .collect();
    Ok(Json(ListLocksResponse::new(locks, next_cursor)))
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
    let user = verify_lock_jwt(repo, headers, &services)?;
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
            LocksProviderError::InvalidId => (StatusCode::BAD_REQUEST, err.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        })?;

    // 3) Return the deleted lock
    let lock = Lock::new(lock.id, lock.path, lock.locked_at, lock.owner.name);
    Ok(Json(DeleteLockResponse::new(lock)))
}
