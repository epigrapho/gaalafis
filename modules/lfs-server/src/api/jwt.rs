use axum::http::{HeaderMap, StatusCode};
use std::{collections::BTreeMap, time::SystemTime};

use crate::traits::token_decoder::TokenDecoder;

#[derive(Debug)]
pub struct Jwt {
    claims: BTreeMap<String, String>,
}

impl Jwt {
    pub fn from_headers(
        headers: HeaderMap,
        decoder: &impl TokenDecoder,
    ) -> Result<Jwt, (StatusCode, String)> {
        if !headers.contains_key("Authorization") {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Authorization header not found".to_string(),
            ));
        }

        let authorization = headers.get("Authorization").unwrap();
        let authorization_str = authorization.to_str().map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Failed to parse Authorization header".to_string(),
            )
        })?;

        let bearer_split = authorization_str.split(' ').collect::<Vec<&str>>();
        let &token = bearer_split.get(1).ok_or((
            StatusCode::UNAUTHORIZED,
            "Failed to parse Authorization header".to_string(),
        ))?;

        return Jwt::from_token(token, decoder);
    }

    fn from_token(token: &str, decoder: &impl TokenDecoder) -> Result<Jwt, (StatusCode, String)> {
        let claims = match decoder.decode_token(token) {
            Ok(c) => c,
            Err(err) => {
                let inner_error_messager = err.to_string();
                let error_message = format!("Failed to decode jwt token {}", &inner_error_messager);
                return Err((StatusCode::UNAUTHORIZED, error_message));
            }
        };

        if Self::is_expired(&claims) {
            return Err((StatusCode::UNAUTHORIZED, String::from("Token expired")));
        }

        Ok(Jwt { claims })
    }

    fn is_expired(claims: &BTreeMap<String, String>) -> bool {
        let exp = claims.get("exp");
        let exp = match exp {
            Some(exp) => exp,
            None => return true,
        };
        let exp = match exp.parse::<u64>() {
            Ok(exp) => exp,
            Err(_) => return true,
        };
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs() > exp,
            Err(_) => true,
        }
    }

    pub fn get_claim(&self, claim: &str) -> Result<String, (StatusCode, String)> {
        let value = self.claims.get(claim);
        match value {
            Some(value) => Ok(value.clone()),
            None => Err((
                StatusCode::UNAUTHORIZED,
                format!("Claim {} not found in token", claim),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RepoTokenPayload {
    repo: String,
    operation: String,
}

impl RepoTokenPayload {
    pub fn new(token: &Jwt) -> Result<RepoTokenPayload, (StatusCode, String)> {
        let repo = token.get_claim("repo")?;
        let operation = token.get_claim("operation")?;

        // Operation should be upload or download
        if operation != "upload" && operation != "download" {
            return Err((
                StatusCode::UNAUTHORIZED,
                String::from("Invalid operation claim in token, must be upload or download"),
            ));
        }

        Ok(RepoTokenPayload {
            repo: repo.to_string(),
            operation: operation.to_string(),
        })
    }

    pub fn has_access(&self, repo: &str) -> bool {
        self.repo == repo
    }

    pub fn has_write_access(&self) -> bool {
        self.operation == "upload"
    }
}

#[cfg(test)]
impl RepoTokenPayload {
    pub fn new_for_test(repo: &str, operation: &str) -> RepoTokenPayload {
        RepoTokenPayload {
            repo: repo.to_string(),
            operation: operation.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::token_decoder::TokenDecoder;
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct JwtTokenDecoderForTest {
        pub resolves_repo: String,
        pub resolves_operation: String,
        pub expires_in: i64,
    }
    impl TokenDecoder for JwtTokenDecoderForTest {
        fn decode_token(
            &self,
            _: &str,
        ) -> Result<std::collections::BTreeMap<String, String>, Box<dyn std::error::Error>>
        {
            let mut claims = std::collections::BTreeMap::new();
            claims.insert("repo".to_string(), self.resolves_repo.clone());
            claims.insert("operation".to_string(), self.resolves_operation.clone());
            let exp = match self.expires_in.signum() {
                -1 => {
                    SystemTime::now() - std::time::Duration::from_secs(self.expires_in.abs() as u64)
                }
                _ => {
                    SystemTime::now() + std::time::Duration::from_secs(self.expires_in.abs() as u64)
                }
            };
            claims.insert(
                "exp".to_string(),
                exp.duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            );
            Ok(claims)
        }
    }

    #[test]
    fn test_new_repo_token_payload() {
        let token = super::Jwt {
            claims: vec![
                ("repo".to_string(), "my-repo".to_string()),
                ("operation".to_string(), "download".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let payload = super::RepoTokenPayload::new(&token).unwrap();
        assert_eq!(payload.repo, "my-repo");
        assert_eq!(payload.operation, "download");
    }

    #[test]
    fn test_new_repo_token_payload_missing_repo() {
        let token = super::Jwt {
            claims: vec![("operation".to_string(), "download".to_string())]
                .into_iter()
                .collect(),
        };
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Claim repo not found in token");
    }

    #[test]
    fn test_new_repo_token_payload_missing_operation() {
        let token = super::Jwt {
            claims: vec![("repo".to_string(), "my-repo".to_string())]
                .into_iter()
                .collect(),
        };
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Claim operation not found in token");
    }

    #[test]
    fn test_new_repo_token_payload_invalid_operation() {
        let token = super::Jwt {
            claims: vec![
                ("repo".to_string(), "my-repo".to_string()),
                ("operation".to_string(), "foo".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(
            err.1,
            "Invalid operation claim in token, must be upload or download"
        );
    }

    #[test]
    fn test_has_access() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            operation: "download".to_string(),
        };
        assert!(payload.has_access("my-repo"));
    }

    #[test]
    fn test_has_access_false() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            operation: "download".to_string(),
        };
        assert!(!payload.has_access("another-repo"));
    }

    #[test]
    fn test_has_write_access() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            operation: "upload".to_string(),
        };
        assert!(payload.has_write_access());
    }

    #[test]
    fn test_has_write_access_false() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            operation: "download".to_string(),
        };
        assert!(!payload.has_write_access());
    }

    #[test]
    fn test_is_expired() {
        let mut claims = std::collections::BTreeMap::new();
        claims.insert(
            "exp".to_string(),
            (SystemTime::now() + std::time::Duration::from_secs(60))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        );
        let jwt = super::Jwt { claims };
        assert!(!super::Jwt::is_expired(&jwt.claims));
    }

    #[test]
    fn test_is_expired_false() {
        let mut claims = std::collections::BTreeMap::new();
        claims.insert(
            "exp".to_string(),
            (SystemTime::now() - std::time::Duration::from_secs(60))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        );
        let jwt = super::Jwt { claims };
        assert!(super::Jwt::is_expired(&jwt.claims));
    }

    #[test]
    fn test_is_expired_no_exp() {
        let claims = std::collections::BTreeMap::new();
        let jwt = super::Jwt { claims };
        assert!(super::Jwt::is_expired(&jwt.claims));
    }

    #[test]
    fn test_from_headers() {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", HeaderValue::from_static("Bearer token"));
        let decoder = JwtTokenDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let jwt = super::Jwt::from_headers(headers, &decoder).unwrap();
        assert_eq!(jwt.claims.len(), 3);
        assert_eq!(jwt.claims.get("repo").unwrap(), "my-repo");
        assert_eq!(jwt.claims.get("operation").unwrap(), "download");
    }

    #[test]
    fn test_from_headers_missing_header() {
        let headers = HeaderMap::new();
        let decoder = JwtTokenDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let err = super::Jwt::from_headers(headers, &decoder).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Authorization header not found");
    }

    #[test]
    fn test_from_headers_malformed_header() {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", HeaderValue::from_static("token"));
        let decoder = JwtTokenDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let err = super::Jwt::from_headers(headers, &decoder).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Failed to parse Authorization header");
    }
}
