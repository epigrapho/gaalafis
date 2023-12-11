use axum::http::StatusCode;
use serde::Deserialize;

use super::jwt::RepoTokenPayload;

#[derive(Deserialize)]
pub struct QueryRepo {
    pub repo: String,
}

impl QueryRepo {
    /// Verify that the repo in the jwt payload match the repo in the query.
    pub fn assert_repo_match_token(
        &self,
        jwt_payload: &RepoTokenPayload,
    ) -> Result<(), (StatusCode, String)> {
        if !jwt_payload.has_access(&self.repo) {
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_repo_match_token() {
        let query = QueryRepo {
            repo: "my-repo".to_string(),
        };
        let jwt_payload = RepoTokenPayload::new_for_test("my-repo", "download");
        let res = query.assert_repo_match_token(&jwt_payload);
        assert!(res.is_ok());
    }

    #[test]
    fn test_assert_repo_match_token_non_matching() {
        let query = QueryRepo {
            repo: "another-repo".to_string(),
        };
        let jwt_payload = RepoTokenPayload::new_for_test("my-repo", "download");
        let err = query.assert_repo_match_token(&jwt_payload).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
        assert_eq!(err.1, "Unauthorized");
    }
}
