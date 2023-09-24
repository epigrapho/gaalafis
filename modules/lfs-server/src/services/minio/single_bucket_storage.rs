use async_trait::async_trait;

use s3::{creds::Credentials, Bucket, Region};

use crate::{
    api::objects_batch::response::ObjectAction,
    traits::file_storage::{
        FileStorageLinkSigner, FileStorageMetaRequester, FileStorageMetaResult,
    },
};

/* -------------------------------------------------------------------------- */
/*                                  Requester                                 */
/* -------------------------------------------------------------------------- */

pub struct MinioSingleBucketStorage {
    bucket: Bucket,
}

impl MinioSingleBucketStorage {
    pub fn new(
        bucket_name: String,
        region: Region,
        credentials: Credentials,
    ) -> MinioSingleBucketStorage {
        // we can't start the server without the bucket
        let bucket: Bucket = Bucket::new(&bucket_name, region, credentials)
            .unwrap()
            .with_path_style();
        MinioSingleBucketStorage { bucket }
    }

    pub fn get_object_path(&self, repo: &str, oid: &str) -> String {
        format!("{}/objects/{}", repo, oid)
    }
}

/* -------------------------------------------------------------------------- */
/*                                    Meta                                    */
/* -------------------------------------------------------------------------- */

#[async_trait]
impl FileStorageMetaRequester for MinioSingleBucketStorage {
    async fn get_meta_result<'a>(&self, repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a> {
        let s3_path = self.get_object_path(repo, oid);
        let meta = self.bucket.head_object(s3_path).await;
        let size = match meta {
            Ok(meta        // Get metadata about the file
) => meta.0.content_length,
            Err(_e) => {
                return FileStorageMetaResult::not_found(repo, oid);
            }
        };

        if let Some(sized) = size {
            return FileStorageMetaResult {
                repo,
                oid,
                exists: true,
                size: if sized > 0 {
                    sized.try_into().unwrap()
                } else {
                    0
                },
            };
        } else {
            return FileStorageMetaResult::not_found(repo, oid);
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                                link signing                                */
/* -------------------------------------------------------------------------- */

#[async_trait]
impl FileStorageLinkSigner for MinioSingleBucketStorage {
    async fn get_presigned_link<'a>(
        &self,
        _result: FileStorageMetaResult<'a>,
    ) -> Result<ObjectAction, Box<dyn std::error::Error>> {
        let s3_path = self.get_object_path(_result.repo, _result.oid);
        let link = self.bucket.presign_get(s3_path, 3600, None)?;
        return Ok(ObjectAction::new(link, None, 3600));
    }

    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        _size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>> {
        let s3_path = self.get_object_path(result.repo, result.oid);
        let link = self.bucket.presign_put(s3_path, 3600, None)?;
        return Ok((ObjectAction::new(link, None, 3600), None));
    }
}

/* -------------------------------------------------------------------------- */
/*                                    tests                                   */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use s3::{creds::Credentials, Region};

    use crate::traits::file_storage::{FileStorageLinkSigner, FileStorageMetaResult, FileStorageMetaRequester};

    use super::MinioSingleBucketStorage;

    fn get_mock_storage() -> MinioSingleBucketStorage {
        let storage = MinioSingleBucketStorage::new(
            String::from("bucket"),
            Region::Custom {
                region: String::from("test"),
                endpoint: String::from("http://localhost:9000"),
            },
            Credentials::new(Some("minio_access_key"), Some("minio_secret_key"), None, None, None).unwrap(),
        );
        return storage;
    }

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_get_object_path() {
        let storage = get_mock_storage();
        let path = storage.get_object_path("repo", "oid");
        assert_eq!(path, "repo/objects/oid");
    }

    #[test]
    fn test_get_signer() {
        let result = FileStorageMetaResult {
            repo: "repo",
            oid: "oid",
            exists: true,
            size: 0,
        };
        let signed = aw!(get_mock_storage().get_presigned_link(result)).unwrap();

        assert!(signed.href.starts_with("http://localhost:9000/bucket/repo/objects/oid?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=minio_access_key"));
        assert!(signed.href.contains("&X-Amz-Expires=3600"));
        assert!(signed.href.contains("&X-Amz-SignedHeaders=host"));
        assert!(signed.href.contains("&X-Amz-Signature="));
        assert!(signed.href.contains("&X-Amz-Date="));
        assert_eq!(signed.expires_in, 3600);
        assert!(signed.header.is_none());
    }

    #[test]
    fn test_put_signer() {
        let result = FileStorageMetaResult {
            repo: "repo",
            oid: "oid",
            exists: true,
            size: 0,
        };
        let (upload, verify) = aw!(get_mock_storage().post_presigned_link(result, 30)).unwrap();

        assert!(upload.href.starts_with("http://localhost:9000/bucket/repo/objects/oid?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=minio_access_key"));
        assert!(upload.href.contains("&X-Amz-Expires=3600"));
        assert!(upload.href.contains("&X-Amz-SignedHeaders=host"));
        assert!(upload.href.contains("&X-Amz-Signature="));
        assert!(upload.href.contains("&X-Amz-Date="));
        assert_eq!(upload.expires_in, 3600);
        assert!(upload.header.is_none());
        assert!(verify.is_none());
    }

    #[test]
    fn test_get_meta_success() {
        let meta = aw!(get_mock_storage().get_meta_result("repo", "test.txt"));
        
        assert!(meta.exists);
        assert_eq!(meta.size, 5);
        assert_eq!(meta.oid, "test.txt");
        assert_eq!(meta.repo, "repo");
    }

    #[test]
    fn test_get_meta_not_found() {
        let meta = aw!(get_mock_storage().get_meta_result("repo", "test_not_found.txt"));
        
        assert!(!meta.exists);
        assert_eq!(meta.size, 0);
        assert_eq!(meta.oid, "test_not_found.txt");
        assert_eq!(meta.repo, "repo");
    }
}