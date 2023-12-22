use crate::traits::file_storage::{
    FileStorageMetaRequester, FileStorageMetaResult, FileStorageProxy,
};
use async_trait::async_trait;
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct LocalFileStorageConfig {
    pub root_path: String,
}

pub struct LocalFileStorage {
    root_path: String,
}

impl LocalFileStorage {
    pub fn new(root_path: String) -> LocalFileStorage {
        LocalFileStorage { root_path }
    }

    pub fn from_config(config: LocalFileStorageConfig) -> LocalFileStorage {
        LocalFileStorage::new(config.root_path)
    }

    pub fn get_object_path(&self, repo: &str, oid: &str) -> String {
        format!("{}/{}/objects/{}", &self.root_path, repo, oid)
    }

    pub fn get_mime_type_object_path(&self, repo: &str, oid: &str) -> String {
        format!("{}/{}/mime-types/{}.mime", &self.root_path, repo, oid)
    }

    async fn create_if_missing(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let exists = tokio::fs::try_exists(path).await?;
        if !exists {
            tokio::fs::create_dir_all(path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl FileStorageMetaRequester for LocalFileStorage {
    async fn get_meta_result<'a>(&self, repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a> {
        if !Regex::new(r"^([a-z0-9\-_]*)\.([a-z0-9\-_]*)$")
            .unwrap()
            .is_match(oid)
        {
            return FileStorageMetaResult::not_found(repo, oid);
        }

        let path = self.get_object_path(repo, oid);
        let meta = tokio::fs::metadata(path).await;

        let size = meta.map_or(None, |m| Some(m.len()));
        self.match_size(size, repo, oid)
    }
}

#[async_trait]
impl FileStorageProxy for LocalFileStorage {
    async fn get(
        &self,
        repo: &str,
        oid: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
        // Read the file
        let path = self.get_object_path(repo, oid);
        let mut file = tokio::fs::File::open(path).await?;
        let mut response = Vec::new();
        file.read_to_end(&mut response).await?;

        // Read the mime type
        let content_type_file_path = self.get_mime_type_object_path(repo, oid);
        let mut content_type_file = tokio::fs::File::open(content_type_file_path).await?;
        let mut content_type = String::new();
        content_type_file.read_to_string(&mut content_type).await?;

        return Ok((response, content_type));
    }

    async fn post(
        &self,
        repo: &str,
        oid: &str,
        data: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.create_if_missing(&self.root_path).await?;
        self.create_if_missing(&format!("{}/{}", &self.root_path, repo))
            .await?;
        self.create_if_missing(&format!("{}/{}/objects", &self.root_path, repo))
            .await?;
        self.create_if_missing(&format!("{}/{}/mime-types", &self.root_path, repo))
            .await?;

        let path = self.get_object_path(repo, oid);
        let mime_type_path = self.get_mime_type_object_path(repo, oid);

        let mut mime_type_file = tokio::fs::File::create(mime_type_path).await?;
        mime_type_file.write_all(_content_type.as_bytes()).await?;

        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(&data).await?;

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_get_object_path() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        let path = storage.get_object_path("repo", "oid");
        assert_eq!(path, format!("/tmp/{}/repo/objects/oid", random_dir));
    }

    #[test]
    fn test_get_meta_result() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        let result = aw!(storage.get_meta_result("repo", "oid"));
        assert_eq!(result.repo, "repo");
        assert_eq!(result.oid, "oid");
        assert!(!result.exists);
        assert_eq!(result.size, 0);
    }

    #[test]
    fn test_post() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        aw!(storage.post("repo", "oid", vec![1, 2, 3], "application/octet-stream")).unwrap();
    }

    #[test]
    fn test_post_and_retrieve() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        aw!(storage.post("repo", "oid", vec![1, 2, 3], "application/octet-stream")).unwrap();
        let retrieved = aw!(storage.get("repo", "oid"));
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().0, vec![1, 2, 3]);
    }

    #[test]
    fn test_post_and_get_meta() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        aw!(storage.post("repo", "oid", vec![1, 2, 3], "application/octet-stream")).unwrap();
        let result = aw!(storage.get_meta_result("repo", "oid"));
        assert!(result.exists);
        assert_eq!(result.size, 3);
        assert_eq!(result.repo, "repo");
        assert_eq!(result.oid, "oid");
    }

    #[test]
    fn test_keep_mime_type() {
        let random_dir = uuid::Uuid::new_v4().to_string();
        let storage = super::LocalFileStorage::new(format!("/tmp/{}", random_dir));
        aw!(storage.post("repo", "oid", vec![1, 2, 3], "image/png")).unwrap();
        let retrieved = aw!(storage.get("repo", "oid"));
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().1, "image/png");
    }
}
