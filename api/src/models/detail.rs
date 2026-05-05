use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SubmissionDetail {
    pub id: String,
    pub submitter_type: String,
    pub submitter_name: Option<String>,
    pub submitter_avatar: Option<String>,
    pub model_id: String,
    pub model_name: String,
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

#[derive(Debug, Serialize)]
pub struct ModelSummary {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub avg_score: f64,
    pub submission_count: u32,
    pub website_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ModelDetailResponse {
    pub model: ModelSummary,
    pub submissions: Vec<SubmissionDetail>,
}
