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
            Operation::Download,
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
