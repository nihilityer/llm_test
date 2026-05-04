mod error;

use crate::error::CodeLlmTestApiError;
use axum::extract::State;
use axum::Json;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_service::Service;
use worker::*;

#[derive(Clone)]
struct AppState {
    database: Arc<D1Database>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestData {
    test: String,
}

fn router(env: Env) -> Result<Router> {
    let app_state = AppState {
        database: Arc::new(env.d1("RANK_DATA")?),
    };
    Ok(Router::new().route("/", get(root)).with_state(app_state))
}

#[event(fetch, respond_with_errors)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router(env)?.call(req).await?)
}

#[worker::send]
pub async fn root(State(state): State<AppState>) -> Result<Json<TestData>, CodeLlmTestApiError> {
    let test = match state
        .database
        .prepare("SELECT 'test——value' AS test;")
        .first::<TestData>(None)
        .await?
    {
        None => return Err(CodeLlmTestApiError::NotFound("test".to_string())),
        Some(test) => test,
    };
    Ok(Json(test))
}
