use crate::traits::token_decoder::TokenDecoder;
use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use jwt::{Header, Token, VerifyWithKey};
use sha2::Sha256;

pub struct JwtTokenDecoder {
    secret: String,
}

impl JwtTokenDecoder {
    pub fn new(secret: String) -> JwtTokenDecoder {
        JwtTokenDecoder { secret }
    }

    pub fn from_env_var(key: &str) -> JwtTokenDecoder {
        let secret = std::env::var(key).unwrap();
        JwtTokenDecoder::new(secret)
    }

    pub fn from_file_env_var(key: &str) -> JwtTokenDecoder {
        let path = std::env::var(key).unwrap();
        let secret = std::fs::read_to_string(path).unwrap();
        JwtTokenDecoder::new(secret)
    }
}

impl TokenDecoder for JwtTokenDecoder {
    fn decode_token(
        &self,
        token_str: &str,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes()).unwrap();
        let token: Token<Header, BTreeMap<String, String>, _> = token_str.verify_with_key(&key)?;
        let claims = token.claims().to_owned();
        Ok(claims)
    }
}
