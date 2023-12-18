use serde::Deserialize;

/// A reference to a git pointer. Not used yet. Might be specified in the body
#[derive(Deserialize)]
pub struct Ref {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateLockPayload {
    pub path: String,
    #[serde(rename = "ref")]
    pub ref_: Option<Ref>,
}

#[derive(Deserialize, Default)]
pub struct ListLocksQuery {
    pub repo: String,
    pub path: Option<String>,
    pub id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<String>,
    pub refspec: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct ListLocksForVerificationPayload {
    pub cursor: Option<String>,
    pub limit: Option<String>,
    #[serde(rename = "ref")]
    pub ref_: Option<Ref>,
}

#[derive(Deserialize, Default)]
pub struct DeleteLockPayload {
    pub force: Option<bool>,
    pub ref_: Option<Ref>,
}
