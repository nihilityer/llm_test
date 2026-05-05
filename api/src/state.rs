use crate::error::{ApiError, Result};
use crate::models::WeightConfig;
use std::sync::Arc;
use worker::{D1Database, Env};

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<D1Database>,
    pub env: Arc<Env>,
}

impl AppState {
    pub fn new(env: &Env, database: D1Database) -> Self {
        Self {
            database: Arc::new(database),
            env: Arc::new(env.clone()),
        }
    }

    /// Get a secret or env var, reading from Secrets API on demand.
    pub fn get_secret_or_var(&self, name: &str) -> Result<String> {
        if let Ok(secret) = self.env.secret(name) {
            return Ok(secret.to_string());
        }
        if let Ok(var) = self.env.var(name) {
            return Ok(var.to_string());
        }
        Err(ApiError::internal(format!(
            "Missing config: {}",
            name
        )))
    }

    pub fn get_weight_config(&self) -> WeightConfig {
        let version_weights: serde_json::Value = self
            .env
            .var("VERSION_WEIGHTS")
            .ok()
            .map(|v| v.to_string())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({"v1": 1.0}));

        let oauth = self
            .env
            .var("SUBMITTER_WEIGHT_OAUTH")
            .ok()
            .map(|v| v.to_string())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);

        let anonymous = self
            .env
            .var("SUBMITTER_WEIGHT_ANONYMOUS")
            .ok()
            .map(|v| v.to_string())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        let current_suite = self
            .env
            .var("CURRENT_TEST_SUITE")
            .ok()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "v1".to_string());

        WeightConfig {
            version_weights,
            submitter_weight_oauth: oauth,
            submitter_weight_anonymous: anonymous,
            current_test_suite: current_suite,
        }
    }
}
