mod auth;
mod db;
mod error;
mod handlers;
mod models;

use crate::models::WeightConfig;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_service::Service;
use worker::*;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<D1Database>,
    pub env: Arc<Env>,
    pub secrets: Arc<std::collections::HashMap<String, String>>,
}

impl AppState {
    fn new(env: &Env, database: D1Database) -> Result<Self> {
        let mut secrets = std::collections::HashMap::new();

        // Load secrets (set via `wrangler secret put`)
        for key in &[
            "JWT_SECRET",
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "TURNSTILE_SECRET_KEY",
        ] {
            if let Ok(secret) = env.secret(key) {
                secrets.insert(key.to_string(), secret.to_string());
            }
        }

        Ok(Self {
            database: Arc::new(database),
            env: Arc::new(env.clone()),
            secrets: Arc::new(secrets),
        })
    }

    pub fn get_secret_or_var(&self, name: &str) -> Result<String> {
        if let Some(val) = self.secrets.get(name) {
            return Ok(val.clone());
        }
        if let Ok(var) = self.env.var(name) {
            return Ok(var.to_string());
        }
        Err(Error::RustError(format!(
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

fn router(env: Env) -> Result<Router> {
    let database = env.d1("RANK_DATA")?;
    let app_state = AppState::new(&env, database)?;

    Ok(Router::new()
        .route("/api/auth/github/login", get(handlers::github_login))
        .route("/api/auth/github/callback", post(handlers::github_callback))
        .route("/api/auth/anonymous", post(handlers::anonymous_auth))
        .route("/api/submissions", post(handlers::submit))
        .route("/api/rankings", get(handlers::rankings))
        .with_state(app_state))
}

#[event(fetch, respond_with_errors)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    // CORS preflight
    if *req.method() == "OPTIONS" {
        let cors_headers = Headers::new();
        cors_headers.set("Access-Control-Allow-Origin", "*")?;
        cors_headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
        cors_headers.set(
            "Access-Control-Allow-Headers",
            "Authorization, X-Anonymous-Token, Content-Type",
        )?;
        cors_headers.set("Access-Control-Max-Age", "86400")?;

        let resp = axum::http::Response::builder()
            .status(204)
            .body(axum::body::Body::empty())
            .map_err(|e| Error::RustError(format!("{}", e)))?;
        return Ok(resp);
    }

    Ok(router(env)?.call(req).await?)
}
