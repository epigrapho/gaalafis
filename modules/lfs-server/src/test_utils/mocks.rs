use crate::api::enums::Operation;
use crate::api::objects_batch::response::ObjectAction;
use crate::services::injected_services::InjectedServices;
use crate::traits::file_storage::{
    FileStorageLinkSigner, FileStorageMetaRequester, FileStorageMetaResult, FileStorageProxy,
};
use crate::traits::token_encoder_decoder::TokenEncoderDecoder;
use async_trait::async_trait;
use axum::http::HeaderMap;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MockFileStorageMetaRequester {
    pub size: u64,
    pub found: bool,
}

#[async_trait]
impl FileStorageMetaRequester for MockFileStorageMetaRequester {
    async fn get_meta_result<'a>(&self, repo: &'a str, oid: &'a str) -> FileStorageMetaResult<'a> {
        if self.found {
            FileStorageMetaResult::new(repo, oid, self.size)
        } else {
            FileStorageMetaResult::not_found(repo, oid)
        }
    }
}

pub struct MockLinkSigner {
    pub with_verify: bool,
    pub check_link_succeed: bool,
}

fn build_action(verb: String, result: &FileStorageMetaResult, size: u32) -> ObjectAction {
    ObjectAction::new(
        format!(
            "https://example.com/{}/{}/{}?size={}",
            verb, result.repo, result.oid, size
        ),
        Some("token"),
        60,
    )
}

#[async_trait]
impl FileStorageLinkSigner for MockLinkSigner {
    async fn get_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
    ) -> Result<ObjectAction, Box<dyn std::error::Error>> {
        Ok(build_action(
            String::from("download"),
            &result,
            u32::try_from(result.size).unwrap(),
        ))
    }

    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>> {
        Ok((
            build_action(String::from("upload"), &result, size),
            if self.with_verify {
                Some(build_action(String::from("verify"), &result, size))
            } else {
                None
            },
        ))
    }

    async fn check_link(
        &self,
        _repo: &str,
        _oid: &str,
        _header: Option<&HeaderMap>,
        _operation: Operation,
    ) -> bool {
        return self.check_link_succeed;
    }
}

pub struct DecodedTokenMock {
    pub repo: String,
    pub operation: Operation,
}
pub struct TokenEncoderDecoderMock {
    pub encoded_token: Option<String>,
    pub decoded: Option<DecodedTokenMock>,
    pub expired: bool,
}

impl TokenEncoderDecoder for TokenEncoderDecoderMock {
    fn encode_token(
        &self,
        _claims: &mut BTreeMap<&str, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self.encoded_token {
            Some(ref token) => Ok(token.clone()),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "TokenEncoderDecoderMock error",
            ))),
        }
    }

    fn decode_token(
        &self,
        _token: &str,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        match self.decoded {
            Some(ref decoded) => Ok(vec![
                ("repo".to_string(), decoded.repo.clone()),
                ("operation".to_string(), decoded.operation.to_string()),
                ("user".to_string(), "user".to_string()),
                (
                    "exp".to_string(),
                    (if self.expired { now - 60 } else { now + 60 }).to_string(),
                ),
            ]
            .into_iter()
            .collect()),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "TokenEncoderDecoderMock error",
            ))),
        }
    }
}

pub struct MockProxy {
    pub get_success: bool,
    pub post_success: bool,
}

#[async_trait]
impl FileStorageProxy for MockProxy {
    async fn get(&self, _repo: &str, _oid: &str) -> Result<(Vec<u8>, String), Box<dyn Error>> {
        if self.get_success {
            Ok((vec![1, 2, 3], String::from("application/octet-stream")))
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "MockProxy error",
            )))
        }
    }

    async fn post(
        &self,
        _repo: &str,
        _oid: &str,
        _data: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), Box<dyn Error>> {
        if self.post_success {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "MockProxy error",
            )))
        }
    }
}

pub struct MockConfig {
    /**
     * Does the FileStorageMetaRequester find the file?
     */
    pub found: bool,

    /**
     * Size returned by the FileStorageMetaRequester
     */
    pub size: u64,

    /**
     * Does the LinkSigner return verify action?
     */
    pub with_verify: bool,

    /**
     * Does the LinkSigner succeed when verifying the link?
     */
    pub check_link_succeed: bool,

    /**
     * Return value of encode_token(). None if it shall fail to encode.
     */
    pub encoded_token: Option<String>,

    /**
     * Content of the decoded token. None if it shall fail to decode.
     */
    pub decoded: Option<DecodedTokenMock>,

    /**
     * Is decoded token expired?
     */
    pub expired: bool,

    /**
     * Is proxy enabled?
     */
    pub proxy_enabled: bool,

    /**
     * Is proxy get request successful?
     */
    pub proxy_get_success: bool,

    /**
     * Is proxy post request successful?
     */
    pub proxy_post_success: bool,
}

impl Default for MockConfig {
    fn default() -> Self {
        MockConfig {
            found: true,
            size: 50,
            with_verify: true,
            check_link_succeed: true,
            encoded_token: Some(String::from("token")),
            decoded: Some(DecodedTokenMock {
                repo: String::from("a/b/c"),
                operation: Operation::Download,
            }),
            expired: false,
            proxy_enabled: false,
            proxy_get_success: true,
            proxy_post_success: true,
        }
    }
}

pub fn get_mock(config: MockConfig) -> InjectedServices {
    InjectedServices {
        file_storage_meta_requester: Arc::new(MockFileStorageMetaRequester {
            found: config.found,
            size: config.size,
        }),
        file_storage_link_signer: Arc::new(MockLinkSigner {
            with_verify: config.with_verify,
            check_link_succeed: config.check_link_succeed,
        }),
        token_encoder_decoder: Arc::new(TokenEncoderDecoderMock {
            encoded_token: config.encoded_token.clone(),
            decoded: config.decoded.as_ref().map(|s| DecodedTokenMock {
                repo: s.repo.clone(),
                operation: match s.operation {
                    Operation::Download => Operation::Download,
                    Operation::Upload => Operation::Upload,
                },
            }),
            expired: config.expired,
        }),
        locks_provider: None,
        file_storage_proxy: if config.proxy_enabled {
            Some(Arc::new(MockProxy {
                get_success: config.proxy_get_success,
                post_success: config.proxy_post_success,
            }))
        } else {
            None
        },
    }
}
