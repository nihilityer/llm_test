mod auth;
mod db;
mod error;
mod handlers;
mod models;
mod state;

pub use state::AppState;

use axum::routing::{get, post};
use axum::Router;
use tower_service::Service;
use worker::*;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeConsoleWriter};

const ALLOWED_ORIGINS: &[&str] = &[
    "https://llmtest.top",
    "https://www.llmtest.top",
    "http://localhost:5173",
];

fn get_allowed_origin(origin: &str) -> Option<&'static str> {
    ALLOWED_ORIGINS
        .iter()
        .find(|&&o| o == origin)
        .copied()
}

fn router(env: Env) -> Result<Router> {
    let database = env.d1("RANK_DATA")?;
    let app_state = AppState::new(&env, database);

    Ok(Router::new()
        .route("/api/auth/github/login", get(handlers::github_login))
        .route("/api/auth/github/callback", post(handlers::github_callback))
        .route("/api/auth/anonymous", post(handlers::anonymous_auth))
        .route("/api/llm-proxy", post(handlers::llm_proxy_handler))
        .route("/api/submissions", post(handlers::submit))
        .route("/api/rankings", get(handlers::rankings))
        .with_state(app_state))
}

#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch, respond_with_errors)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    tracing::info!("Incoming request: {}", req.method().as_ref());

    // Capture Origin before req is consumed by the router
    let req_origin = req
        .headers()
        .get("Origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // CORS preflight
    if *req.method() == "OPTIONS" {
        tracing::debug!("CORS preflight, origin={}", req_origin);

        if let Some(allowed) = get_allowed_origin(&req_origin) {
            let resp = axum::http::Response::builder()
                .status(204)
                .header("Access-Control-Allow-Origin", allowed)
                .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                .header(
                    "Access-Control-Allow-Headers",
                    "Authorization, X-Anonymous-Token, Content-Type",
                )
                .header("Access-Control-Max-Age", "86400")
                .body(axum::body::Body::empty())
                .map_err(|e| Error::RustError(format!("{}", e)))?;
            return Ok(resp);
        }

        let resp = axum::http::Response::builder()
            .status(204)
            .body(axum::body::Body::empty())
            .map_err(|e| Error::RustError(format!("{}", e)))?;
        return Ok(resp);
    }

    let mut resp = router(env)?.call(req).await?;

    // Add CORS header to actual responses
    if let Some(allowed) = get_allowed_origin(&req_origin) {
        resp.headers_mut().insert(
            "Access-Control-Allow-Origin",
            axum::http::HeaderValue::from_str(allowed).unwrap(),
        );
    }

    Ok(resp)
}
