use serde::{Deserialize, Serialize};

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
    pub domains: String, // JSON array
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRow {
    pub id: String,
    pub name: String,
    pub aliases: String, // JSON array
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteModelRow {
    pub website_id: String,
    pub model_id: String,
    pub model_weight: f64,
    pub website_weight: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmissionRow {
    pub id: String,
    pub submitter_type: String,
    pub user_id: Option<String>,
    pub ip_hash: Option<String>,
    pub website_id: String,
    pub model_id: String,
    pub test_suite_version: String,
    pub api_style: String,
    pub endpoint_hash: String,
    pub total_score: f64,
    pub dimension_scores: String,
    pub test_results: String,
    pub created_at: String,
}
