use crate::auth;
use crate::db;
use crate::error::ApiError;
use crate::models::*;
use crate::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use tracing::{debug, info, warn};

/// Extract authentication info from request headers.
/// Returns (submitter_type, user_id, ip_hash).
pub async fn extract_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<(String, Option<String>, Option<String>), ApiError> {
    // Check Authorization: Bearer <jwt>
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ").trim();
                let jwt_secret = state.get_secret_or_var("JWT_SECRET").await?;
                let claims = auth::verify_jwt(token, &jwt_secret)
                    .map_err(|_| ApiError::Unauthorized("Invalid JWT token".into()))?;
                debug!("OAuth auth extracted: user={}", claims.sub);
                return Ok((claims.submitter_type, Some(claims.sub), claims.ip_hash));
            }
        }
    }

    // Check X-Anonymous-Token
    if let Some(anon) = headers.get("X-Anonymous-Token") {
        if let Ok(token) = anon.to_str() {
            let jwt_secret = state.get_secret_or_var("JWT_SECRET").await?;
            let claims = auth::verify_jwt(token, &jwt_secret)
                .map_err(|_| ApiError::Unauthorized("Invalid anonymous token".into()))?;
            if claims.submitter_type != "anonymous" {
                warn!("Anonymous token has wrong submitter_type");
                return Err(ApiError::Unauthorized("Invalid anonymous token".into()));
            }
            debug!("Anonymous auth extracted: ip_hash={}", claims.sub);
            return Ok(("anonymous".to_string(), None, Some(claims.sub)));
        }
    }

    debug!("No auth headers found");
    Err(ApiError::Unauthorized("Authentication required".into()))
}

// ---------- GitHub OAuth ----------

#[worker::send]
pub async fn github_login(State(state): State<AppState>) -> Result<Response, ApiError> {
    info!("GitHub login flow initiated");
    let client_id = state
        .get_secret_or_var("GITHUB_CLIENT_ID")
        .await
        .map_err(|_| ApiError::OAuth("GITHUB_CLIENT_ID not configured".into()))?;

    let redirect_uri = state
        .get_secret_or_var("GITHUB_REDIRECT_URI")
        .await
        .ok()
        .or_else(|| {
            state.env.var("FRONTEND_URL").ok().map(|url| {
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
    info!("GitHub OAuth callback received");
    let client_id = state.get_secret_or_var("GITHUB_CLIENT_ID").await?;
    let client_secret = state.get_secret_or_var("GITHUB_CLIENT_SECRET").await?;
    let jwt_secret = state.get_secret_or_var("JWT_SECRET").await?;

    let (github_id, login, avatar_url) =
        auth::exchange_github_code(&body.code, &client_id, &client_secret).await?;

    let user = match db::find_user_by_github_id(&state.database, github_id).await? {
        Some(existing) => {
            debug!("Existing GitHub user found: {}", existing.login);
            existing
        }
        None => {
            debug!("Creating new user for GitHub login: {}", login);
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
    info!("Anonymous auth attempt");
    let turnstile_secret = state.get_secret_or_var("TURNSTILE_SECRET_KEY").await?;

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
        warn!("Anonymous IP rate limit hit: hash={}", &ip_hash);
        return Err(ApiError::RateLimited(
            "Too many submissions from this IP. Please try again later or log in with GitHub."
                .into(),
        ));
    }

    let jwt_secret = state.get_secret_or_var("JWT_SECRET").await?;
    let token = auth::create_anonymous_jwt(&ip_hash, &jwt_secret)?;

    Ok(Json(AuthResponse { token, user: None }))
}
