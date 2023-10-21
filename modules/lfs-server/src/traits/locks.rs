use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use async_trait::async_trait;
use crate::api::locks::response::LockOwner;

pub struct Lock {
    pub id: String,
    pub path: String,
    pub ref_name: String,
    pub owner: LockOwner,
    pub locked_at: SystemTime,
}

#[derive(Debug)]
pub enum LocksProviderError {
    ConnectionFailure(Box<dyn std::error::Error + Send>),
    RequestPreparationFailure(Box<dyn std::error::Error + Send>),
    RequestExecutionFailure(Box<dyn std::error::Error + Send>),
    ParsingResponseDataFailure(Box<dyn std::error::Error + Send>),
    InvalidId,
    InvalidLimit,
    InvalidCursor,
    LockNotFound,
    LockAlreadyExists,
}

impl Display for LocksProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocksProviderError::ConnectionFailure(err) => write!(f, "ConnectionFailure: {}", err),
            LocksProviderError::RequestPreparationFailure(err) => write!(f, "RequestPreparationFailure: {}", err),
            LocksProviderError::RequestExecutionFailure(err) => write!(f, "RequestExecutionFailure: {}", err),
            LocksProviderError::ParsingResponseDataFailure(err) => write!(f, "ParsingResponseDataFailure: {}", err),
            LocksProviderError::InvalidId => write!(f, "InvalidId"),
            LocksProviderError::InvalidLimit => write!(f, "InvalidLimit"),
            LocksProviderError::InvalidCursor => write!(f, "InvalidCursor"),
            LocksProviderError::LockNotFound => write!(f, "LockNotFound"),
            LocksProviderError::LockAlreadyExists => write!(f, "LockAlreadyExists"),
        }
    }
}

#[async_trait]
pub trait LocksProvider: Sync + Send {
    async fn create_lock(
        &self,
        repo: &str,
        user_name: &str,
        path: &str,
        ref_name: Option<&str>,
    ) -> Result<(Lock, bool), LocksProviderError>;
    async fn list_locks(
        &self,
        repo: &str,
        path: Option<&str>,
        id: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u64>,
        ref_name: Option<&str>,
    ) -> Result<(Option<String>, Vec<Lock>), LocksProviderError>;
    async fn delete_lock(
        &self,
        repo: &str,
        user_name: &str,
        id: &str,
        ref_name: Option<&str>,
        force: Option<bool>,
    ) -> Result<Lock, LocksProviderError>;
}
