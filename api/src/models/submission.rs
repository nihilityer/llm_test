use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubmissionRequest {
    pub domain: String,
    pub model: String,
    pub test_suite_version: String,
    pub api_style: String,
    pub endpoint_hash: String,
    pub total_score: f64,
    pub dimension_scores: serde_json::Value,
    pub test_results: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub struct SubmissionResponse {
    pub id: String,
    pub website_id: String,
    pub website_name: String,
    pub model_id: String,
    pub model_name: String,
}
