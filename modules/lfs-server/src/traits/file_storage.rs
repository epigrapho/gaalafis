use async_trait::async_trait;

use crate::api::{enums::Operation,objects_batch::response::ObjectAction};

#[derive(Debug)]
pub struct FileStorageMetaResult<'a> {
    pub repo: &'a str,
    pub oid: &'a str,
    pub exists: bool,
    pub size: u64,
}

impl FileStorageMetaResult<'_> {
    pub fn not_found<'a>(repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a> {
        FileStorageMetaResult {
            repo,
            oid,
            exists: false,
            size: 0,
        }
    }

    pub fn new<'a>(repo: &'a str, oid: &'a str, size: u64) -> FileStorageMetaResult<'a> {
        FileStorageMetaResult {
            repo,
            oid,
            exists: true,
            size: if size > 0 { size } else { 0 },
        }
    }
}

#[async_trait]
pub trait FileStorageMetaRequester {
    async fn get_meta_result<'a>(&self, repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a>;
}

#[async_trait]
pub trait FileStorageLinkSigner {
    async fn get_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
    ) -> Result<ObjectAction, Box<dyn std::error::Error>>;
    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait FileStorageProxy {
    async fn check_link(&self, link: &str, operation: Operation) -> bool;
    async fn get(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn post(&self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
}
