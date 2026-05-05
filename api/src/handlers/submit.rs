use crate::db;
use crate::error::ApiError;
use crate::handlers::auth::extract_auth;
use crate::models::*;
use crate::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use tracing::{info, warn};

#[worker::send]
pub async fn submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubmissionRequest>,
) -> Result<Json<SubmissionResponse>, ApiError> {
    info!("Submission received: domain={}, model={}", body.domain, body.model);
    let current_suite = state
        .env
        .var("CURRENT_TEST_SUITE")
        .ok()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "v1".to_string());
    if body.test_suite_version != current_suite {
        warn!(
            "Test suite version mismatch: requested={}, current={}",
            body.test_suite_version, current_suite
        );
        return Err(ApiError::BadRequest(format!(
            "Test suite version mismatch. Current: {}",
            current_suite
        )));
    }

    // Extract auth info
    let (submitter_type, user_id, ip_hash) =
        extract_auth(&headers, &state).await?;

    // Rate limiting for anonymous
    if submitter_type == "anonymous" {
        let ih = ip_hash.as_deref().unwrap_or("");
        let website = db::find_website_by_domain(&state.database, &body.domain).await?;
        if let Some(ref ws) = website {
            let same_site = db::count_anonymous_submissions_by_ip_and_website(
                &state.database,
                ih,
                &ws.id,
                24,
            )
            .await?;
            if same_site >= 3 {
                warn!("Anonymous per-website rate limit hit: ip={}, website={}", ih, &ws.id);
                return Err(ApiError::RateLimited(
                    "Too many submissions to this website from this IP.".into(),
                ));
            }
        }
    }

    // Find or create website
    let website = match db::find_website_by_domain(&state.database, &body.domain).await? {
        Some(ws) => ws,
        None => {
            let ws_id = uuid::Uuid::new_v4().to_string();
            db::create_website(
                &state.database,
                &ws_id,
                &body.domain,
                std::slice::from_ref(&body.domain),
            )
            .await?
        }
    };

    // Resolve model
    let model_name = body.model.trim();
    let model = match db::find_model_by_name_or_alias(&state.database, model_name).await? {
        Some(m) => m,
        None => {
            let model_id = uuid::Uuid::new_v4().to_string();
            db::create_model(&state.database, &model_id, model_name).await?
        }
    };

    // Ensure website-model link exists
    if db::find_website_model(&state.database, &website.id, &model.id)
        .await?
        .is_none()
    {
        db::create_website_model(&state.database, &website.id, &model.id).await?;
    }

    // Upsert submission
    let sub_id = uuid::Uuid::new_v4().to_string();
    let dimension_scores_str = serde_json::to_string(&body.dimension_scores)?;
    let test_results_str = serde_json::to_string(&body.test_results)?;

    db::upsert_submission(
        &state.database,
        &sub_id,
        &submitter_type,
        &user_id,
        &ip_hash,
        &website.id,
        &model.id,
        &body.test_suite_version,
        &body.api_style,
        &body.endpoint_hash,
        body.total_score,
        &dimension_scores_str,
        &test_results_str,
    )
    .await?;

    info!("Submission persisted: id={}, website={}, model={}", sub_id, &website.name, &model.name);
    Ok(Json(SubmissionResponse {
        id: sub_id,
        website_id: website.id,
        website_name: website.name,
        model_id: model.id,
        model_name: model.name,
    }))
}
