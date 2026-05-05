#[derive(Debug, Clone)]
pub struct WeightConfig {
    pub version_weights: serde_json::Value,
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
