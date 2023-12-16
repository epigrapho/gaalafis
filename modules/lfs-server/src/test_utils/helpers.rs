use crate::api::enums::{HashAlgorithm, Operation, Transfer};
use crate::api::objects_batch::body::{ObjectIdentity, ObjectsBatchRequestPayload};
use crate::api::repo_query::QueryRepo;
use axum::http::{HeaderMap, HeaderValue};

#[macro_export]
macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

impl QueryRepo {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }
}

impl ObjectIdentity {
    pub fn new(oid: &str, size: u32) -> Self {
        Self {
            oid: oid.to_string(),
            size,
        }
    }
}

impl ObjectsBatchRequestPayload {
    pub fn new_download_default(objects: Vec<ObjectIdentity>) -> Self {
        Self {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Basic]),
            objects,
            hash_algo: HashAlgorithm::Sha256,
        }
    }

    pub fn new_upload_default(objects: Vec<ObjectIdentity>) -> Self {
        Self {
            operation: Operation::Upload,
            transfers: Some(vec![Transfer::Basic]),
            objects,
            hash_algo: HashAlgorithm::Sha256,
        }
    }
}

pub fn test_auth_headers(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.append("Authorization", HeaderValue::from_str(value).unwrap());
    headers
}
