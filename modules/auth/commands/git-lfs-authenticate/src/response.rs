use serde::Serialize;
use std::{collections::BTreeMap, fmt::{Display, Formatter, Error}};

#[derive(Serialize)]
pub struct AuthResponse {
    href: String,
    header: BTreeMap<String, String>,
    expires_in: u64,
}

impl AuthResponse {
    pub fn new(href: String, token: String, expires_in: u64) -> AuthResponse {
        let mut header = BTreeMap::new();
        header.insert("Authorization".to_string(), format!("Bearer {}", token));
        AuthResponse {
            href,
            header,
            expires_in,
        }
    }
}

impl Display for AuthResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let json = serde_json::to_string(self).map_err(|_| Error)?;
        write!(f, "{}", json)
    }
}
