use crate::traits::token_encoder_decoder::TokenEncoderDecoder;
use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;

pub struct JwtTokenEncoderDecoder {
    secret: String,
    expires_in: u64,
}

impl JwtTokenEncoderDecoder {
    pub fn new(secret: String, expires_in: u64) -> JwtTokenEncoderDecoder {
        JwtTokenEncoderDecoder { secret, expires_in }
    }

    pub fn from_env_var(key: &str, expires_in_key: &str) -> JwtTokenEncoderDecoder {
        let secret = std::env::var(key).unwrap();
        let expires_in = std::env::var(expires_in_key)
            .unwrap()
            .parse::<u64>()
            .unwrap();
        JwtTokenEncoderDecoder::new(secret, expires_in)
    }

    pub fn from_file_env_var(key: &str, expires_in_key: &str) -> JwtTokenEncoderDecoder {
        let path = std::env::var(key).unwrap();
        let secret = std::fs::read_to_string(path).unwrap();
        let expires_in = std::env::var(expires_in_key)
            .unwrap()
            .parse::<u64>()
            .unwrap();
        JwtTokenEncoderDecoder::new(secret, expires_in)
    }
}

impl TokenEncoderDecoder for JwtTokenEncoderDecoder {
    fn decode_token(
        &self,
        token_str: &str,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes())?;
        let token: Token<Header, BTreeMap<String, String>, _> = token_str.verify_with_key(&key)?;
        let claims = token.claims().to_owned();
        Ok(claims)
    }

    fn encode_token(
        &self,
        claims: &mut BTreeMap<&str, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes())?;

        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(self.expires_in as i64))
            .unwrap()
            .timestamp();

        claims.insert("exp", exp.to_string());
        let token_str = claims.sign_with_key(&key)?;
        Ok(token_str)
    }
}
