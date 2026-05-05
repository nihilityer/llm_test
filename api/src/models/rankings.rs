use serde::Serialize;

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
pub struct ModelRankingEntry {
    pub rank: u32,
    pub model_id: String,
    pub model_name: String,
    pub aliases: Vec<String>,
    pub avg_score: f64,
    pub submission_count: u32,
    pub max_score: f64,
    pub min_score: f64,
    pub last_tested_at: String,
    pub website_count: u32,
}

#[derive(Debug, Serialize)]
pub struct RankingsResponse<T: Serialize> {
    pub rankings: Vec<T>,
    pub total: u32,
}
