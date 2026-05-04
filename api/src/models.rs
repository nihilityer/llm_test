use serde::{Deserialize, Serialize};

// ---------- Auth ----------

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
    pub sub: String,       // user_id for oauth, ip_hash for anonymous
    pub login: Option<String>,
    pub submitter_type: String,  // "oauth" | "anonymous"
    pub ip_hash: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

// ---------- Submission ----------

#[derive(Debug, Deserialize)]
pub struct SubmissionRequest {
    pub domain: String,
    pub test_suite_version: String,
    pub api_style: String,
    pub endpoint_hash: String,
    pub total_score: f64,
    pub dimension_scores: serde_json::Value,  // JSON object
    pub test_results: serde_json::Value,       // JSON array
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: String,
    pub website_id: String,
    pub website_name: String,
}

// ---------- Database row types ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRow {
    pub id: String,
    pub github_id: Option<i64>,
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteRow {
    pub id: String,
    pub name: String,
    pub domains: String,  // JSON array
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmissionRow {
    pub id: String,
    pub submitter_type: String,
    pub user_id: Option<String>,
    pub ip_hash: Option<String>,
    pub website_id: String,
    pub test_suite_version: String,
    pub api_style: String,
    pub endpoint_hash: String,
    pub total_score: f64,
    pub dimension_scores: String,  // JSON
    pub test_results: String,       // JSON
    pub created_at: String,
}

// ---------- Ranking response ----------

#[derive(Debug, Serialize)]
pub struct RankingEntry {
    pub rank: u32,
    pub website_id: String,
    pub website_name: String,
    pub domains: Vec<String>,
    pub avg_score: f64,
    pub submission_count: u32,
    pub max_score: f64,
    pub min_score: f64,
    pub last_tested_at: String,
}

#[derive(Debug, Serialize)]
pub struct RankingsResponse {
    pub rankings: Vec<RankingEntry>,
    pub total: u32,
}

// ---------- Website detail response ----------

#[derive(Debug, Serialize)]
pub struct SubmissionDetail {
    pub id: String,
    pub submitter_type: String,
    pub submitter_name: Option<String>,
    pub submitter_avatar: Option<String>,
    pub total_score: f64,
    pub dimension_scores: serde_json::Value,
    pub test_suite_version: String,
    pub api_style: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WebsiteDetailResponse {
    pub website: WebsiteSummary,
    pub submissions: Vec<SubmissionDetail>,
}

#[derive(Debug, Serialize)]
pub struct WebsiteSummary {
    pub id: String,
    pub name: String,
    pub domains: Vec<String>,
    pub avg_score: f64,
    pub submission_count: u32,
}

// ---------- Weight config (parsed from env vars) ----------

#[derive(Debug, Clone)]
pub struct WeightConfig {
    pub version_weights: serde_json::Value,  // {"v1": 0.7, "v2": 1.0}
    pub submitter_weight_oauth: f64,
    pub submitter_weight_anonymous: f64,
    pub current_test_suite: String,
}

impl WeightConfig {
    pub fn get_submitter_weight(&self, submitter_type: &str) -> f64 {
        match submitter_type {
            "oauth" => self.submitter_weight_oauth,
            "anonymous" => self.submitter_weight_anonymous,
            _ => 0.7,
        }
    }

    pub fn get_version_weight(&self, version: &str) -> f64 {
        self.version_weights
            .get(version)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5)
    }
}
