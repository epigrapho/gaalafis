use axum::http::StatusCode;

use crate::services::jwt::Jwt;

#[derive(Debug)]
pub struct RepoTokenPayload {
    repo: String,
    pub user: String,
    operation: String,
}

impl RepoTokenPayload {
    pub fn new(token: &Jwt) -> Result<RepoTokenPayload, (StatusCode, String)> {
        let repo = token.get_claim("repo")?;
        let user = token.get_claim("user")?;
        let operation = token.get_claim("operation")?;

        // Operation should be upload or download
        if operation != "upload" && operation != "download" {
            return Err((
                StatusCode::UNAUTHORIZED,
                String::from("Invalid operation claim in token, must be upload or download"),
            ));
        }

        Ok(RepoTokenPayload {
            repo,
            user,
            operation,
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
            user: String::from("John Doe"),
            operation: operation.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::services::jwt::Jwt;

    #[test]
    fn test_new_repo_token_payload() {
        let token = Jwt::new_for_test(
            vec![
                ("repo".to_string(), "my-repo".to_string()),
                ("user".to_string(), "John Doe".to_string()),
                ("operation".to_string(), "download".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        let payload = super::RepoTokenPayload::new(&token).unwrap();
        assert_eq!(payload.repo, "my-repo");
        assert_eq!(payload.operation, "download");
    }

    #[test]
    fn test_new_repo_token_payload_missing_repo() {
        let token = Jwt::new_for_test(
            vec![("operation".to_string(), "download".to_string())]
                .into_iter()
                .collect(),
        );
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Claim repo not found in token");
    }

    #[test]
    fn test_new_repo_token_payload_missing_user() {
        let token = Jwt::new_for_test(
            vec![("repo".to_string(), "my-repo".to_string())]
                .into_iter()
                .collect(),
        );
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Claim user not found in token");
    }

    #[test]
    fn test_new_repo_token_payload_missing_operation() {
        let token = Jwt::new_for_test(
            vec![
                ("repo".to_string(), "my-repo".to_string()),
                ("user".to_string(), "John Doe".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        let err = super::RepoTokenPayload::new(&token).unwrap_err();
        assert_eq!(err.0, axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Claim operation not found in token");
    }

    #[test]
    fn test_new_repo_token_payload_invalid_operation() {
        let token = Jwt::new_for_test(
            vec![
                ("repo".to_string(), "my-repo".to_string()),
                ("user".to_string(), "John Doe".to_string()),
                ("operation".to_string(), "foo".to_string()),
            ]
            .into_iter()
            .collect(),
        );
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
            user: "John Doe".to_string(),
            operation: "download".to_string(),
        };
        assert!(payload.has_access("my-repo"));
    }

    #[test]
    fn test_has_access_false() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            user: "John Doe".to_string(),
            operation: "download".to_string(),
        };
        assert!(!payload.has_access("another-repo"));
    }

    #[test]
    fn test_has_write_access() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            user: "John Doe".to_string(),
            operation: "upload".to_string(),
        };
        assert!(payload.has_write_access());
    }

    #[test]
    fn test_has_write_access_false() {
        let payload = super::RepoTokenPayload {
            repo: "my-repo".to_string(),
            user: "John Doe".to_string(),
            operation: "download".to_string(),
        };
        assert!(!payload.has_write_access());
    }
}
