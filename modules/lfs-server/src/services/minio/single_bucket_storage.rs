use async_trait::async_trait;

use axum::http::HeaderMap;
use regex::Regex;
use s3::{creds::Credentials, Bucket, Region};

use crate::{
    api::{enums::Operation, objects_batch::response::ObjectAction},
    traits::file_storage::{
        FileStorageLinkSigner, FileStorageMetaRequester, FileStorageMetaResult, FileStorageProxy,
    },
};

/* -------------------------------------------------------------------------- */
/*                                  Requester                                 */
/* -------------------------------------------------------------------------- */

pub struct MinioSingleBucketStorage {
    bucket_direct_access: Bucket,
    bucket_public_access: Bucket,
}

pub struct MinioSingleBucketStorageConfig {
    pub bucket_name: String,
    pub credentials: Credentials,
    pub direct_access_region: Region,
    pub public_access_region: Option<Region>,
}

impl MinioSingleBucketStorage {
    pub fn new(
        bucket_name: String,
        credentials: Credentials,
        direct_access_region: Region,
        public_access_region: Option<Region>,
    ) -> MinioSingleBucketStorage {
        // we can't start the server without the bucket
        let public_access_credentials = credentials.clone();
        let bucket_direct_access: Bucket =
            Bucket::new(&bucket_name, direct_access_region, credentials)
                .unwrap()
                .with_path_style();

        match public_access_region {
            Some(region) => {
                let bucket_public_access: Bucket =
                    Bucket::new(&bucket_name, region, public_access_credentials)
                        .unwrap()
                        .with_path_style();
                Self::new_with_buckets(bucket_direct_access, Some(bucket_public_access))
            }
            None => Self::new_with_buckets(bucket_direct_access, None),
        }
    }

    pub fn new_with_buckets(
        bucket_direct_access: Bucket,
        bucket_public_access: Option<Bucket>,
    ) -> MinioSingleBucketStorage {
        let bucket_public_access = bucket_public_access.unwrap_or(bucket_direct_access.clone());
        MinioSingleBucketStorage {
            bucket_direct_access,
            bucket_public_access,
        }
    }

    pub fn from_config(config: MinioSingleBucketStorageConfig) -> MinioSingleBucketStorage {
        Self::new(
            config.bucket_name,
            config.credentials,
            config.direct_access_region,
            config.public_access_region,
        )
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
        if !Regex::new(r"^([a-z0-9\-_]*)\.([a-z0-9\-_]*)$")
            .unwrap()
            .is_match(oid)
        {
            return FileStorageMetaResult::not_found(repo, oid);
        }

        let s3_path = self.get_object_path(repo, oid);
        let meta = self.bucket_direct_access.head_object(s3_path).await;
        let size = meta
            .map(|m| {
                m.0.content_length
                    .map(|c| u64::try_from(c).map_or(None, Some))
            })
            .unwrap_or(None)
            .flatten();
        return self.match_size(size, repo, oid);
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
        let link = self.bucket_public_access.presign_get(s3_path, 3600, None)?;
        return Ok(ObjectAction::new(link, None, 3600));
    }

    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        _size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>> {
        let s3_path = self.get_object_path(result.repo, result.oid);
        let link = self.bucket_public_access.presign_put(s3_path, 3600, None)?;
        return Ok((ObjectAction::new(link, None, 3600), None));
    }

    async fn check_link(
        &self,
        _repo: &str,
        _oid: &str,
        _header: Option<&HeaderMap>,
        _operation: Operation,
    ) -> bool {
        // in this strategy, we are not responsible for checking the link, it should be done directly by minio
        return false;
    }
}

/* -------------------------------------------------------------------------- */
/*                             Upload and download                            */
/* -------------------------------------------------------------------------- */

#[async_trait]
impl FileStorageProxy for MinioSingleBucketStorage {
    async fn get(
        &self,
        repo: &str,
        oid: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
        let s3_path = self.get_object_path(repo, oid);
        let response = self.bucket_direct_access.get_object(s3_path).await?;
        let content_type = response
            .headers()
            .get("content-type")
            .unwrap_or(&String::from("application/octet-stream"))
            .to_owned();
        return Ok((response.to_vec(), content_type));
    }

    async fn post(
        &self,
        repo: &str,
        oid: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let s3_path = self.get_object_path(repo, oid);
        self.bucket_direct_access
            .put_object_with_content_type(s3_path, &data, content_type)
            .await?;
        return Ok(());
    }
}

/* -------------------------------------------------------------------------- */
/*                                    tests                                   */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};

    use crate::traits::file_storage::{
        FileStorageLinkSigner, FileStorageMetaRequester, FileStorageMetaResult,
    };

    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    async fn init_random_bucket() -> (String, Credentials, Region) {
        let bucket_name = uuid::Uuid::new_v4().to_string();
        let region = Region::Custom {
            region: String::from("test"),
            endpoint: String::from("http://localhost:9000"),
        };
        let credentials = Credentials::new(
            Some("minio_access_key"),
            Some("minio_secret_key"),
            None,
            None,
            None,
        )
        .unwrap();
        let bucket = Bucket::create_with_path_style(
            &bucket_name,
            region.clone(),
            credentials.clone(),
            BucketConfiguration::private(),
        )
        .await
        .unwrap()
        .bucket
        .with_path_style();
        bucket
            .put_object_with_content_type(
                String::from("repo/objects/test.txt"),
                b"hello",
                "text/plain",
            )
            .await
            .unwrap();
        (bucket_name, credentials, region)
    }

    fn get_test_minio_single_bucket_storage(
        bucket_name: String,
        credentials: Credentials,
        region: Region,
    ) -> MinioSingleBucketStorage {
        MinioSingleBucketStorage::new(
            bucket_name.clone(),
            credentials,
            region,
            Some(Region::Custom {
                region: String::from("test"),
                endpoint: String::from("https://storage"),
            }),
        )
    }

    fn get_random_initialized_storage() -> (String, MinioSingleBucketStorage) {
        let (bucket_name, credentials, region) = aw!(init_random_bucket());
        let storage =
            get_test_minio_single_bucket_storage(bucket_name.clone(), credentials, region);
        (bucket_name, storage)
    }

    #[test]
    fn test_get_object_path() {
        let (_, storage) = get_random_initialized_storage();
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
        let (bucket_name, storage) = get_random_initialized_storage();
        let signed = aw!(storage.get_presigned_link(result)).unwrap();

        let expected = format!("https://storage/{}/repo/objects/oid?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=minio_access_key", bucket_name);
        assert!(signed.href.starts_with(&expected));
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
        let (bucket_name, storage) = get_random_initialized_storage();
        let (upload, verify) = aw!(storage.post_presigned_link(result, 30)).unwrap();

        let expected = format!("https://storage/{}/repo/objects/oid?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=minio_access_key", bucket_name);
        assert!(upload.href.starts_with(&expected));
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
        let (_, storage) = get_random_initialized_storage();
        let meta = aw!(storage.get_meta_result("repo", "test.txt"));

        assert!(meta.exists);
        assert_eq!(meta.size, 5);
        assert_eq!(meta.oid, "test.txt");
        assert_eq!(meta.repo, "repo");
    }

    #[test]
    fn test_get_meta_not_found() {
        let (_, storage) = get_random_initialized_storage();
        let meta = aw!(storage.get_meta_result("repo", "test_not_found.txt"));

        assert!(!meta.exists);
        assert_eq!(meta.size, 0);
        assert_eq!(meta.oid, "test_not_found.txt");
        assert_eq!(meta.repo, "repo");
    }

    #[test]
    fn test_get_success() {
        let (_, storage) = get_random_initialized_storage();
        let (data, content_type) = aw!(storage.get("repo", "test.txt")).unwrap();
        assert_eq!(data, b"hello");
        assert_eq!(content_type, "text/plain");
    }

    #[test]
    fn test_get_not_found() {
        let (_, storage) = get_random_initialized_storage();
        let result = aw!(storage.get("repo", "test_not_found.txt"));
        let error = result.unwrap_err();
        assert!(error.to_string().starts_with("Got HTTP 404 with content"));
    }

    #[test]
    fn test_post_success() {
        let (_, storage) = get_random_initialized_storage();
        let result =
            aw!(storage.post("repo", "another-test.txt", b"hello2".to_vec(), "text/plain"));
        assert!(result.is_ok());

        let response = aw!(storage
            .bucket_direct_access
            .get_object("/repo/objects/another-test.txt"))
        .unwrap();
        let headers = response.headers();
        let content_type = headers.get("content-type").unwrap();
        assert_eq!(response.to_vec(), b"hello2");
        assert_eq!(content_type, "text/plain");
    }

    #[test]
    fn test_post_wrong_bucket() {
        let (_, credentials, region) = aw!(init_random_bucket());
        let storage =
            get_test_minio_single_bucket_storage(String::from("other-bucket"), credentials, region);
        let result =
            aw!(storage.post("repo", "another-test.txt", b"hello2".to_vec(), "text/plain"));
        let error = result.unwrap_err();
        assert!(error.to_string().starts_with("Got HTTP 404 with content"));
    }
}
