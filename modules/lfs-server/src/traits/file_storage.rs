use async_trait::async_trait;
use axum::http::HeaderMap;

use crate::api::{enums::Operation, objects_batch::response::ObjectAction};

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
pub trait FileStorageMetaRequester: Sync + Send {
    async fn get_meta_result<'a>(&self, repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a>;

    fn match_size<'a>(
        &self,
        size: Option<u64>,
        repo: &'a str,
        oid: &'a str,
    ) -> FileStorageMetaResult<'a> {
        size.map_or(FileStorageMetaResult::not_found(repo, oid), |s| {
            FileStorageMetaResult {
                repo,
                oid,
                exists: true,
                size: s,
            }
        })
    }
}

#[async_trait]
pub trait FileStorageLinkSigner: Sync + Send {
    async fn get_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
    ) -> Result<ObjectAction, Box<dyn std::error::Error>>;
    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>>;
    async fn check_link(
        &self,
        repo: &str,
        oid: &str,
        header: Option<&HeaderMap>,
        operation: Operation,
    ) -> bool;
}

#[async_trait]
pub trait FileStorageProxy: Sync + Send {
    async fn get(
        &self,
        repo: &str,
        oid: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>>;
    async fn post(
        &self,
        repo: &str,
        oid: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
