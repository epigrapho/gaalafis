use std::{collections::BTreeMap, time::SystemTime};

use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;

pub enum JwtSignError {
    SystemTime,
    SecretDecoding,
    JwtSigning,
}

pub struct JwtPayload<'a> {
    repo: &'a str,
    user: &'a str,
    operation: &'a str,
}

impl JwtPayload<'_> {
    pub fn new<'a>(repo: &'a str, user: &'a str, operation: &'a str) -> JwtPayload<'a> {
        JwtPayload {
            repo,
            user,
            operation,
        }
    }

    pub fn sign(&self, jwt_secret: &str, expires_in: &u64) -> Result<String, JwtSignError> {
        let exp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| JwtSignError::SystemTime)?
            .as_secs()
            + expires_in;
        let exp = (1000 * exp).to_string();

        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes())
            .map_err(|_| JwtSignError::SecretDecoding)?;

        let mut claims = BTreeMap::new();
        claims.insert("repo".to_string(), self.repo);
        claims.insert("user".to_string(), self.user);
        claims.insert("operation".to_string(), self.operation);
        claims.insert("exp".to_string(), exp.as_str());
        claims
            .sign_with_key(&key)
            .map_err(|_| JwtSignError::JwtSigning)
    }
}
