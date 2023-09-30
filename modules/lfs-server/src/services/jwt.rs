use axum::http::{HeaderMap, StatusCode};
use std::{collections::BTreeMap, time::SystemTime};

use crate::traits::token_encoder_decoder::TokenEncoderDecoder;

#[derive(Debug)]
pub struct Jwt {
    claims: BTreeMap<String, String>,
}

impl Jwt {
    pub fn from_headers(
        headers: &HeaderMap,
        decoder: &dyn TokenEncoderDecoder,
    ) -> Result<Jwt, (StatusCode, String)> {
        let &token = headers
            .get("Authorization")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Authorization header not found".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Failed to parse Authorization header".to_string(),
                )
            })?
            .split(' ')
            .collect::<Vec<&str>>()
            .get(1)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Failed to parse Authorization header".to_string(),
            ))?;

        Jwt::from_token(token, decoder)
    }

    fn from_token(
        token: &str,
        decoder: &dyn TokenEncoderDecoder,
    ) -> Result<Jwt, (StatusCode, String)> {
        let claims = decoder.decode_token(token).map_err(|err| {
            let inner_error_messager = err.to_string();
            let error_message = format!("Failed to decode jwt token {}", &inner_error_messager);
            (StatusCode::UNAUTHORIZED, error_message)
        })?;

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

#[cfg(test)]
impl Jwt {
    pub fn new_for_test(claims: BTreeMap<String, String>) -> Jwt {
        Jwt { claims }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::token_encoder_decoder::TokenEncoderDecoder;
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct JwtTokenEncoderDecoderForTest {
        pub resolves_repo: String,
        pub resolves_operation: String,
        pub expires_in: i64,
    }
    impl TokenEncoderDecoder for JwtTokenEncoderDecoderForTest {
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

        fn encode_token(
            &self,
            _claims: &mut std::collections::BTreeMap<&str, String>,
        ) -> Result<String, Box<dyn std::error::Error>> {
            panic!("Not implemented")
        }
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
        let decoder = JwtTokenEncoderDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let jwt = super::Jwt::from_headers(&headers, &decoder).unwrap();
        assert_eq!(jwt.claims.len(), 3);
        assert_eq!(jwt.claims.get("repo").unwrap(), "my-repo");
        assert_eq!(jwt.claims.get("operation").unwrap(), "download");
    }

    #[test]
    fn test_from_headers_missing_header() {
        let headers = HeaderMap::new();
        let decoder = JwtTokenEncoderDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let err = super::Jwt::from_headers(&headers, &decoder).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Authorization header not found");
    }

    #[test]
    fn test_from_headers_malformed_header() {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", HeaderValue::from_static("token"));
        let decoder = JwtTokenEncoderDecoderForTest {
            resolves_repo: "my-repo".to_string(),
            resolves_operation: "download".to_string(),
            expires_in: 60,
        };
        let err = super::Jwt::from_headers(&headers, &decoder).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Failed to parse Authorization header");
    }
}
