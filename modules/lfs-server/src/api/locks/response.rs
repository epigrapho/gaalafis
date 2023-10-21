use std::time::SystemTime;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct LockOwner {
    pub name: String,
}

#[derive(Serialize)]
pub struct Lock {
    id: String,
    path: String,
    #[serde(serialize_with = "iso_8601")]
    locked_at: SystemTime,
    owner: LockOwner
}

impl Lock {
    pub fn new(id: String, path: String, locked_at: SystemTime, owner_name: String) -> Self {
        Lock {
            id,
            path,
            locked_at,
            owner: LockOwner {
                name: owner_name,
            }
        }
    }
}

#[derive(Serialize)]
pub struct CreateLockResponse {
    lock: Lock,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl CreateLockResponse {
    pub fn new(lock: Lock) -> Self {
        CreateLockResponse { lock, message: None }
    }
    pub fn with_message(lock: Lock, message: String) -> Self {
        CreateLockResponse { lock, message: Some(message) }
    }
}

#[derive(Serialize)]
pub struct ListLocksResponse {
    locks: Vec<Lock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

impl ListLocksResponse {
    pub fn new(locks: Vec<Lock>, next_cursor: Option<String>) -> Self {
        ListLocksResponse { locks, next_cursor }
    }
}

#[derive(Serialize)]
pub struct ListLocksForVerificationResponse {
    ours: Vec<Lock>,
    theirs: Vec<Lock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

#[derive(Serialize)]
pub struct DeleteLockResponse {
    lock: Lock,
}


impl DeleteLockResponse {
    pub fn new(lock: Lock) -> Self {
        DeleteLockResponse { lock }
    }
}

pub fn iso_8601<S>(st: &SystemTime, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let dt: DateTime<Utc> = (*st).into();
    let serialized = dt.to_rfc3339_opts(SecondsFormat::Secs, false);
    s.serialize_str(&serialized)
}

