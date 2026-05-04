use crate::auth;
use crate::db;
use crate::error::ApiError;
use crate::models::*;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;
use worker::*;

// ---------- GitHub OAuth ----------

#[worker::send]
pub async fn github_login(State(state): State<AppState>) -> Result<Response, ApiError> {
    let client_id = state
        .secrets
        .get("GITHUB_CLIENT_ID")
        .cloned()
        .or_else(|| {
            state.env.var("GITHUB_CLIENT_ID").ok().map(|v| v.to_string())
        })
        .ok_or_else(|| ApiError::OAuth("GITHUB_CLIENT_ID not configured".into()))?;

    let redirect_uri = state
        .secrets
        .get("GITHUB_REDIRECT_URI")
        .cloned()
        .or_else(|| {
            state
                .env
                .var("FRONTEND_URL")
                .ok()
                .map(|url| {
                    let s = url.to_string();
                    format!("{}/auth/callback", s.trim_end_matches('/'))
                })
        })
        .unwrap_or_else(|| "http://localhost:5173/auth/callback".to_string());

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user",
        client_id, redirect_uri
    );
    Ok(Redirect::to(&url).into_response())
}

#[worker::send]
pub async fn github_callback(
    State(state): State<AppState>,
    Json(body): Json<GithubCallbackRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let client_id = state.get_secret_or_var("GITHUB_CLIENT_ID")?;
    let client_secret = state.get_secret_or_var("GITHUB_CLIENT_SECRET")?;
    let jwt_secret = state.get_secret_or_var("JWT_SECRET")?;

    let (github_id, login, avatar_url) =
        auth::exchange_github_code(&body.code, &client_id, &client_secret).await?;

    let user = match db::find_user_by_github_id(&state.database, github_id).await? {
        Some(existing) => existing,
        None => {
            let user_id = uuid::Uuid::new_v4().to_string();
            db::create_user(&state.database, &user_id, github_id, &login, &avatar_url).await?;
            UserRow {
                id: user_id,
                github_id: Some(github_id),
                login: login.clone(),
                avatar_url: avatar_url.clone(),
            }
        }
    };

    let token = auth::create_oauth_jwt(&user.id, &user.login, &jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: Some(UserInfo {
            id: user.id,
            login: user.login,
            avatar_url: user.avatar_url,
        }),
    }))
}

// ---------- Anonymous Turnstile Auth ----------

#[worker::send]
pub async fn anonymous_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AnonymousRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let turnstile_secret = state.get_secret_or_var("TURNSTILE_SECRET_KEY")?;

    let verified = auth::verify_turnstile(&body.turnstile_token, &turnstile_secret).await?;
    if !verified {
        return Err(ApiError::BadRequest("Turnstile verification failed".into()));
    }

    let ip = headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0.0.0.0");
    let ip_hash = auth::hash_ip(ip);

    let count_site = db::count_anonymous_submissions_by_ip(&state.database, &ip_hash, 24).await?;
    if count_site >= 10 {
        return Err(ApiError::RateLimited(
            "Too many submissions from this IP. Please try again later or log in with GitHub."
                .into(),
        ));
    }

    let jwt_secret = state.get_secret_or_var("JWT_SECRET")?;
    let token = auth::create_anonymous_jwt(&ip_hash, &jwt_secret)?;

    Ok(Json(AuthResponse { token, user: None }))
}

// ---------- Submit ----------

#[worker::send]
pub async fn submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubmissionRequest>,
) -> Result<Json<SubmissionResponse>, ApiError> {
    let current_suite = state
        .env
        .var("CURRENT_TEST_SUITE")
        .ok()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "v1".to_string());
    if body.test_suite_version != current_suite {
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
        &body.test_suite_version,
        &body.api_style,
        &body.endpoint_hash,
        body.total_score,
        &dimension_scores_str,
        &test_results_str,
    )
    .await?;

    Ok(Json(SubmissionResponse {
        id: sub_id,
        website_id: website.id,
        website_name: website.name,
    }))
}

// ---------- Rankings (three-in-one) ----------

#[derive(Debug, Deserialize, Default)]
pub struct RankingsQuery {
    pub website_id: Option<String>,
    pub search: Option<String>,
    pub style: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[worker::send]
pub async fn rankings(
    State(state): State<AppState>,
    Query(query): Query<RankingsQuery>,
) -> Result<Response, ApiError> {
    // Website detail mode
    if let Some(ref website_id) = query.website_id {
        return website_detail(&state, website_id)
            .await
            .map(|r| r.into_response());
    }

    // Rankings list mode
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let submissions = db::get_all_submissions_for_rankings(
        &state.database,
        query.style.as_deref(),
    )
    .await?;
    let websites = db::get_all_websites(&state.database).await?;

    // Build website_id -> Vec<submission>
    let mut site_subs: HashMap<String, Vec<&SubmissionRow>> = HashMap::new();
    for sub in &submissions {
        site_subs
            .entry(sub.website_id.clone())
            .or_default()
            .push(sub);
    }

    let weight_config = state.get_weight_config();

    let mut entries: Vec<RankingEntry> = Vec::new();
    for ws in &websites {
        let subs = site_subs.get(&ws.id);

        // Filter by search
        if let Some(ref search) = query.search {
            let search_lower = search.to_lowercase();
            let name_match = ws.name.to_lowercase().contains(&search_lower);
            let domain_match = ws.domains.to_lowercase().contains(&search_lower);
            if !name_match && !domain_match {
                continue;
            }
        }

        let sub_count = subs.map(|s| s.len()).unwrap_or(0) as u32;
        if sub_count == 0 {
            continue;
        }

        let subs = subs.unwrap();
        let weighted_scores: Vec<f64> = subs
            .iter()
            .map(|s| {
                let vw = weight_config.get_version_weight(&s.test_suite_version);
                let sw = weight_config.get_submitter_weight(&s.submitter_type);
                s.total_score * vw * sw
            })
            .collect();

        let sum: f64 = weighted_scores.iter().sum();
        let avg_score = sum / weighted_scores.len() as f64;
        let max_score = weighted_scores
            .iter()
            .fold(f64::MIN, |a, &b| if b > a { b } else { a });
        let min_score = weighted_scores
            .iter()
            .fold(f64::MAX, |a, &b| if b < a { b } else { a });
        let last_tested = subs
            .iter()
            .map(|s| s.created_at.clone())
            .max()
            .unwrap_or_default();

        let domains: Vec<String> =
            serde_json::from_str(&ws.domains).unwrap_or_else(|_| vec![ws.name.clone()]);

        entries.push(RankingEntry {
            rank: 0,
            website_id: ws.id.clone(),
            website_name: ws.name.clone(),
            domains,
            avg_score: (avg_score * 100.0).round() / 100.0,
            submission_count: sub_count,
            max_score: (max_score * 100.0).round() / 100.0,
            min_score: (min_score * 100.0).round() / 100.0,
            last_tested_at: last_tested,
        });
    }

    entries.sort_by(|a, b| {
        b.avg_score
            .partial_cmp(&a.avg_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = entries.len() as u32;

    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = (i + 1 + offset as usize) as u32;
    }

    let paged: Vec<RankingEntry> = entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(Json(RankingsResponse {
        rankings: paged,
        total,
    })
    .into_response())
}

async fn website_detail(
    state: &AppState,
    website_id: &str,
) -> Result<Json<WebsiteDetailResponse>, ApiError> {
    let website = db::find_website_by_id(&state.database, website_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Website not found".into()))?;

    let submissions = db::get_submissions_by_website(&state.database, website_id).await?;

    let user_ids: Vec<String> = submissions
        .iter()
        .filter(|s| s.submitter_type == "oauth")
        .filter_map(|s| s.user_id.clone())
        .collect();

    let users = db::get_all_oauth_users(&state.database, &user_ids).await?;

    let weight_config = state.get_weight_config();
    let sub_count = submissions.len() as u32;

    let weighted_scores: Vec<f64> = submissions
        .iter()
        .map(|s| {
            let vw = weight_config.get_version_weight(&s.test_suite_version);
            let sw = weight_config.get_submitter_weight(&s.submitter_type);
            s.total_score * vw * sw
        })
        .collect();
    let total: f64 = weighted_scores.iter().sum();
    let avg_score = if !weighted_scores.is_empty() {
        total / weighted_scores.len() as f64
    } else {
        0.0
    };

    let domains: Vec<String> =
        serde_json::from_str(&website.domains).unwrap_or_else(|_| vec![website.name.clone()]);

    let submission_details: Vec<SubmissionDetail> = submissions
        .iter()
        .map(|s| {
            let (name, avatar) = s
                .user_id
                .as_ref()
                .and_then(|uid| users.get(uid))
                .map(|u| (Some(u.login.clone()), u.avatar_url.clone()))
                .unwrap_or((None, None));

            SubmissionDetail {
                id: s.id.clone(),
                submitter_type: s.submitter_type.clone(),
                submitter_name: name,
                submitter_avatar: avatar,
                total_score: s.total_score,
                dimension_scores: serde_json::from_str(&s.dimension_scores).unwrap_or_default(),
                test_suite_version: s.test_suite_version.clone(),
                api_style: s.api_style.clone(),
                created_at: s.created_at.clone(),
            }
        })
        .collect();

    Ok(Json(WebsiteDetailResponse {
        website: WebsiteSummary {
            id: website.id,
            name: website.name,
            domains,
            avg_score: (avg_score * 100.0).round() / 100.0,
            submission_count: sub_count,
        },
        submissions: submission_details,
    }))
}

// ---------- Auth helpers ----------

async fn extract_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<(String, Option<String>, Option<String>), ApiError> {
    // Check Authorization: Bearer <jwt>
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ").trim();
                let jwt_secret = state.get_secret_or_var("JWT_SECRET")?;
                let claims = auth::verify_jwt(token, &jwt_secret)
                    .map_err(|_| ApiError::Unauthorized("Invalid JWT token".into()))?;
                return Ok((claims.submitter_type, Some(claims.sub), claims.ip_hash));
            }
        }
    }

    // Check X-Anonymous-Token
    if let Some(anon) = headers.get("X-Anonymous-Token") {
        if let Ok(token) = anon.to_str() {
            let jwt_secret = state.get_secret_or_var("JWT_SECRET")?;
            let claims = auth::verify_jwt(token, &jwt_secret)
                .map_err(|_| ApiError::Unauthorized("Invalid anonymous token".into()))?;
            if claims.submitter_type != "anonymous" {
                return Err(ApiError::Unauthorized("Invalid anonymous token".into()));
            }
            return Ok(("anonymous".to_string(), None, Some(claims.sub)));
        }
    }

    Err(ApiError::Unauthorized("Authentication required".into()))
}
