use crate::error::{ApiError, Result};
use crate::models::JwtClaims;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use tracing::{error, warn};
use worker::{Date, Fetch, Headers, Method, Request, RequestInit};
use worker::wasm_bindgen::JsValue;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_jwt(claims: &JwtClaims, secret: &str) -> Result<String> {
    let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
    let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(claims)?.as_bytes());
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| ApiError::internal("HMAC key creation failed"))?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes().as_slice());

    Ok(format!("{}.{}", signing_input, signature))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<JwtClaims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(ApiError::internal("Invalid JWT format"));
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    let signing_input = format!("{}.{}", header_b64, payload_b64);

    let signature = URL_SAFE_NO_PAD
        .decode(signature_b64.as_bytes())
        .map_err(|e| ApiError::internal(format!("Base64 decode error: {}", e)))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| ApiError::internal("HMAC key creation failed"))?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&signature).map_err(|_| {
        warn!("JWT signature verification failed for sub");
        ApiError::internal("JWT signature verification failed")
    })?;

    let payload_json = URL_SAFE_NO_PAD
        .decode(payload_b64.as_bytes())
        .map_err(|e| ApiError::internal(format!("Base64 decode error: {}", e)))?;
    let payload_str = String::from_utf8(payload_json)
        .map_err(|e| ApiError::internal(format!("UTF-8 decode error: {}", e)))?;

    let claims: JwtClaims = serde_json::from_str(&payload_str)
        .map_err(|e| ApiError::internal(format!("JSON parse error: {}", e)))?;

    // Check expiration
    let now_ms = Date::now().as_millis();
    if claims.exp * 1000 < now_ms {
        warn!("JWT expired for sub={}", claims.sub);
        return Err(ApiError::internal("JWT expired"));
    }

    Ok(claims)
}

fn current_timestamp() -> u64 {
    Date::now().as_millis() / 1000
}

pub fn create_oauth_jwt(user_id: &str, login: &str, secret: &str) -> Result<String> {
    let now = current_timestamp();
    let claims = JwtClaims {
        sub: user_id.to_string(),
        login: Some(login.to_string()),
        submitter_type: "oauth".to_string(),
        ip_hash: None,
        iat: now,
        exp: now + 30 * 24 * 3600, // 30 days
    };
    sign_jwt(&claims, secret)
}

pub fn create_anonymous_jwt(ip_hash: &str, secret: &str) -> Result<String> {
    let now = current_timestamp();
    let claims = JwtClaims {
        sub: ip_hash.to_string(),
        login: None,
        submitter_type: "anonymous".to_string(),
        ip_hash: Some(ip_hash.to_string()),
        iat: now,
        exp: now + 24 * 3600, // 24 hours
    };
    sign_jwt(&claims, secret)
}

pub fn hash_ip(ip: &str) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn make_headers(map: &[(&str, &str)]) -> Result<Headers> {
    let headers = Headers::new();
    for (k, v) in map {
        headers.set(k, v)?;
    }
    Ok(headers)
}

/// Exchange GitHub OAuth code for an access token, then fetch user info.
/// Returns (github_id, login, avatar_url).
pub async fn exchange_github_code(
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<(i64, String, Option<String>)> {
    // Step 1: Exchange code for access token
    let token_url = "https://github.com/login/oauth/access_token";
    let token_body = serde_json::json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "code": code,
    });

    let mut init = RequestInit::new();
    init.method = Method::Post;
    init.headers = make_headers(&[
        ("Accept", "application/json"),
        ("Content-Type", "application/json"),
    ])?;
    init.body = Some(JsValue::from_str(&token_body.to_string()));

    let req = Request::new_with_init(token_url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;

    let resp_json: serde_json::Value = resp.json().await?;
    let access_token = resp_json["access_token"]
        .as_str()
        .ok_or_else(|| {
            error!("GitHub OAuth: failed to get access_token");
            ApiError::OAuth("Failed to get access_token from GitHub".into())
        })?
        .to_string();

    // Step 2: Fetch user info
    let mut user_init = RequestInit::new();
    user_init.method = Method::Get;
    user_init.headers = make_headers(&[
        ("Authorization", &format!("Bearer {}", access_token)),
        ("User-Agent", "code-llm-test"),
    ])?;

    let user_req = Request::new_with_init("https://api.github.com/user", &user_init)?;
    let mut user_resp = Fetch::Request(user_req).send().await?;

    let user_json: serde_json::Value = user_resp.json().await?;
    let github_id = user_json["id"]
        .as_i64()
        .ok_or_else(|| ApiError::OAuth("Failed to get github id".into()))?;
    let login = user_json["login"]
        .as_str()
        .ok_or_else(|| ApiError::OAuth("Failed to get github login".into()))?
        .to_string();
    let avatar_url = user_json["avatar_url"].as_str().map(|s| s.to_string());

    Ok((github_id, login, avatar_url))
}

/// Verify a Cloudflare Turnstile token.
pub async fn verify_turnstile(token: &str, secret_key: &str) -> Result<bool> {
    let url = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
    let body = serde_json::json!({
        "secret": secret_key,
        "response": token,
    });

    let mut init = RequestInit::new();
    init.method = Method::Post;
    init.headers = make_headers(&[("Content-Type", "application/json")])?;
    init.body = Some(JsValue::from_str(&body.to_string()));

    let req = Request::new_with_init(url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;

    let resp_json: serde_json::Value = resp.json().await?;
    let success = resp_json["success"].as_bool().unwrap_or(false);
    if !success {
        warn!("Turnstile verification failed");
    }
    Ok(success)
}
