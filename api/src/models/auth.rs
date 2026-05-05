use serde::{Deserialize, Serialize};

// ---------- Auth request/response ----------

#[derive(Debug, Deserialize)]
pub struct GithubCallbackRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct AnonymousRequest {
    pub turnstile_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub login: String,
    pub avatar_url: Option<String>,
}

// ---------- JWT Claims ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub login: Option<String>,
    pub submitter_type: String,
    pub ip_hash: Option<String>,
    pub exp: u64,
    pub iat: u64,
}
