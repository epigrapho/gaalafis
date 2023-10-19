use std::time::SystemTime;

use async_trait::async_trait;

pub struct LockOwner {
    pub name: String,
}

pub struct Lock {
    pub id: String,
    pub path: String,
    pub ref_name: String,
    pub owner: LockOwner,
    pub locked_at: SystemTime,
}

#[derive(Debug)]
pub enum LocksProviderError {
    ConnectionFailure(Box<dyn std::error::Error>),
    RequestPreparationFailure(Box<dyn std::error::Error>),
    RequestExecutionFailure(Box<dyn std::error::Error>),
    ParsingResponseDataFailure(Box<dyn std::error::Error>),
    InvalidId,
    InvalidLimit,
    InvalidCursor,
}

#[async_trait]
pub trait LocksProvider: Sync + Send {
    async fn create_lock(
        &self,
        repo: &str,
        user_name: &str,
        path: &str,
        ref_name: &str,
    ) -> Result<String, LocksProviderError>;
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
