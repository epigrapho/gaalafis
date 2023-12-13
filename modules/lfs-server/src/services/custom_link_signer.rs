use std::collections::BTreeMap;

use async_trait::async_trait;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::{
    api::{enums::Operation, objects_batch::response::ObjectAction},
    services::jwt::Jwt,
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaResult},
        token_encoder_decoder::TokenEncoderDecoder,
    },
};

#[derive(Serialize, Deserialize)]
struct LinkSignature {
    operation: Operation,
    oid: String,
    repo: String,
}

impl LinkSignature {
    pub fn new(operation: Operation, oid: String, repo: String) -> LinkSignature {
        LinkSignature {
            operation,
            oid,
            repo,
        }
    }

    pub fn from_headers(headers: &HeaderMap, signer: &impl TokenEncoderDecoder) -> LinkSignature {
        let jwt = Jwt::from_headers(headers, signer).unwrap();
        let operation = match jwt.get_claim("operation").unwrap().as_str() {
            "download" => Operation::Download,
            "upload" => Operation::Upload,
            _ => panic!(),
        };
        let oid = jwt.get_claim("oid").unwrap();
        let repo = jwt.get_claim("repo").unwrap();
        LinkSignature {
            operation,
            oid,
            repo,
        }
    }

    pub fn sign(&self, signer: &impl TokenEncoderDecoder) -> String {
        let mut claims = BTreeMap::new();
        claims.insert("operation", self.operation.to_string());
        claims.insert("oid", self.oid.clone());
        claims.insert("repo", self.repo.clone());
        signer.encode_token(&mut claims).unwrap()
    }
}

pub struct CustomLinkSigner<TTokenEncoderDecoder: TokenEncoderDecoder> {
    host: String,
    signer: TTokenEncoderDecoder,
}

impl<TTokenEncoderDecoder: TokenEncoderDecoder> CustomLinkSigner<TTokenEncoderDecoder> {
    pub fn new(
        host: String,
        signer: TTokenEncoderDecoder,
    ) -> CustomLinkSigner<TTokenEncoderDecoder> {
        CustomLinkSigner { host, signer }
    }

    pub fn from_env_var(
        host_key: &str,
        signer: TTokenEncoderDecoder,
    ) -> CustomLinkSigner<TTokenEncoderDecoder> {
        let host = std::env::var(host_key).unwrap();
        CustomLinkSigner::new(host, signer)
    }
}

#[async_trait]
impl<TTokenEncoderDecoder: TokenEncoderDecoder + Sync + Send> FileStorageLinkSigner
    for CustomLinkSigner<TTokenEncoderDecoder>
{
    async fn get_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
    ) -> Result<ObjectAction, Box<dyn std::error::Error>> {
        let link = format!(
            "{}/{}/objects/access/{}",
            self.host, result.repo, result.oid
        );
        let signature = LinkSignature::new(
            Operation::Download,
            result.oid.to_string(),
            result.repo.to_string(),
        );
        return Ok(ObjectAction::new(
            link,
            Some(format!("Bearer {}", signature.sign(&self.signer)).as_str()),
            3600,
        ));
    }

    async fn post_presigned_link<'a>(
        &self,
        result: FileStorageMetaResult<'a>,
        _size: u32,
    ) -> Result<(ObjectAction, Option<ObjectAction>), Box<dyn std::error::Error>> {
        let link = format!(
            "{}/{}/objects/access/{}",
            self.host, result.repo, result.oid
        );
        let signature = LinkSignature::new(
            Operation::Upload,
            result.oid.to_string(),
            result.repo.to_string(),
        );
        return Ok((
            ObjectAction::new(
                link,
                Some(&format!("Bearer {}", signature.sign(&self.signer))),
                3600,
            ),
            None,
        ));
    }

    async fn check_link(
        &self,
        repo: &str,
        oid: &str,
        headers: Option<&HeaderMap>,
        operation: Operation,
    ) -> bool {
        let headers = match headers {
            None => return false,
            Some(s) => s,
        };
        let signature_payload = LinkSignature::from_headers(headers, &self.signer);
        return signature_payload.oid == oid
            && signature_payload.repo == repo
            && signature_payload.operation == operation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::jwt_token_encoder_decoder::JwtTokenEncoderDecoder;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    fn parse_header_helper(link: ObjectAction) -> BTreeMap<String, String> {
        let authorization = link.header.unwrap().authorization;
        assert!(authorization.starts_with("Bearer"));
        let token = authorization.split(' ').collect::<Vec<&str>>()[1];
        let encoder_decoder = JwtTokenEncoderDecoder::new("secret".to_string(), 3600);
        encoder_decoder.decode_token(token).unwrap()
    }

    fn get_signer() -> CustomLinkSigner<JwtTokenEncoderDecoder> {
        let encoder_decoder = JwtTokenEncoderDecoder::new("secret".to_string(), 3600);
        CustomLinkSigner::new("http://localhost:8080".to_string(), encoder_decoder)
    }

    #[test]
    fn test_get_presigned_link() {
        // Build link
        let link =
            aw!(get_signer().get_presigned_link(FileStorageMetaResult::new("repo", "oid", 100)))
                .unwrap();
        assert_eq!(link.href, "http://localhost:8080/repo/objects/access/oid");

        // Check headers
        let jwt = parse_header_helper(link);
        assert_eq!(jwt.get("oid").unwrap(), "oid");
        assert_eq!(jwt.get("repo").unwrap(), "repo");
        assert_eq!(jwt.get("operation").unwrap(), "download");
    }

    #[test]
    fn test_post_presigned_link() {
        // Build link
        let (post_link, verify_link) =
            aw!(get_signer()
                .post_presigned_link(FileStorageMetaResult::new("repo", "oid", 100), 100))
            .unwrap();
        assert!(verify_link.is_none());
        assert_eq!(
            post_link.href,
            "http://localhost:8080/repo/objects/access/oid"
        );

        // Check headers
        let jwt = parse_header_helper(post_link);
        assert_eq!(jwt.get("oid").unwrap(), "oid");
        assert_eq!(jwt.get("repo").unwrap(), "repo");
        assert_eq!(jwt.get("operation").unwrap(), "upload");
    }

    fn get_test_headers(operation: Operation) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let encoder_decoder = JwtTokenEncoderDecoder::new("secret".to_string(), 3600);
        let mut claims = vec![
            ("oid", "oid"),
            ("repo", "repo"),
            ("operation", operation.to_string().as_str()),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect::<BTreeMap<&str, String>>();
        let jwt = encoder_decoder.encode_token(&mut claims).unwrap();
        let authorization = format!("Bearer {}", jwt);
        headers.insert("Authorization", authorization.parse().unwrap());
        headers
    }

    #[test]
    fn test_check_link_success() {
        assert!(aw!(get_signer().check_link(
            "repo",
            "oid",
            Some(&get_test_headers(Operation::Download)),
            Operation::Download
        )));
    }

    #[test]
    fn test_check_link_bad_repo() {
        assert!(!aw!(get_signer().check_link(
            "bad-repo",
            "oid",
            Some(&get_test_headers(Operation::Download)),
            Operation::Download
        )));
    }

    #[test]
    fn test_check_link_bad_oid() {
        assert!(!aw!(get_signer().check_link(
            "repo",
            "bad-oid",
            Some(&get_test_headers(Operation::Download)),
            Operation::Download
        )));
    }

    #[test]
    fn test_check_link_operation_too_low() {
        assert!(!aw!(get_signer().check_link(
            "repo",
            "oid",
            Some(&get_test_headers(Operation::Download)),
            Operation::Upload
        )));
    }

    #[test]
    fn test_check_link_operation_too_high() {
        assert!(!aw!(get_signer().check_link(
            "repo",
            "oid",
            Some(&get_test_headers(Operation::Upload)),
            Operation::Download
        )));
    }


}
