use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use serde::Deserialize;
use std::collections::HashMap;
use worker::*;

use crate::error::{ApiError, Result};
use crate::AppState;

#[derive(Deserialize)]
pub struct ProxyRequest {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Hop-by-hop headers that should NOT be forwarded
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "transfer-encoding",
    "host",
    "origin",
    "referer",
    "content-encoding",
    "content-length",
    "access-control-allow-origin",
];

fn redact_headers_for_log(headers: &HashMap<String, String>) -> HashMap<String, &str> {
    let sensitive: &[&str] = &["authorization", "x-api-key", "x-goog-api-key", "api-key"];
    let mut result = HashMap::new();
    for (k, v) in headers {
        if sensitive.iter().any(|s| k.to_lowercase() == *s) {
            result.insert(k.clone(), "[REDACTED]");
        } else {
            result.insert(k.clone(), v.as_str());
        }
    }
    result
}

fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.iter().any(|h| *h == name.to_lowercase())
}

fn is_private_host(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("://localhost")
        || lower.contains("://127.0.0.1")
        || lower.contains("://[::1]")
        || lower.contains("://0.0.0.0")
        || lower.contains("://169.254.")
        || lower.contains("://10.")
        || lower.contains("://172.16.")
        || lower.contains("://192.168.")
}

fn parse_method(method: &str) -> core::result::Result<Method, ApiError> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(Method::Get),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "PATCH" => Ok(Method::Patch),
        "DELETE" => Ok(Method::Delete),
        _ => Err(ApiError::BadRequest(format!(
            "Unsupported method: {}",
            method
        ))),
    }
}

#[worker::send]
#[axum::debug_handler]
pub async fn llm_proxy_handler(
    _state: State<AppState>,
    Json(payload): Json<ProxyRequest>,
) -> Result<Response> {
    tracing::info!(
        "LLM proxy: method={}, url={}, headers={:?}",
        payload.method,
        payload.url,
        redact_headers_for_log(&payload.headers),
    );

    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err(ApiError::BadRequest(
            "Invalid URL scheme in proxy request".into(),
        ));
    }

    if is_private_host(&payload.url) {
        return Err(ApiError::BadRequest(
            "Proxying to private/internal hosts is not allowed".into(),
        ));
    }

    let http_method = parse_method(&payload.method)?;

    // Build upstream request
    let mut init = RequestInit::new();
    init.method = http_method;
    init.headers = Headers::new();

    for (key, value) in &payload.headers {
        if is_hop_by_hop(key) {
            continue;
        }
        init.headers.set(key, value)?;
    }

    if let Some(ref body_str) = payload.body {
        init.body = Some(wasm_bindgen::JsValue::from_str(body_str));
    }

    let req = Request::new_with_init(&payload.url, &init)?;
    let mut worker_resp = Fetch::Request(req).send().await?;

    let upstream_status = worker_resp.status_code();
    tracing::info!("LLM proxy upstream: status={}", upstream_status);

    // Capture response headers before consuming the response body
    let mut resp_headers: Vec<(String, String)> = Vec::new();
    for (k, v) in worker_resp.headers().entries() {
        resp_headers.push((k, v));
    }

    // Buffer response body (ByteStream is not Send so can't use Body::from_stream)
    let body_bytes = worker_resp.bytes().await?;
    let axum_body = Body::from(Bytes::from(body_bytes));

    let status =
        StatusCode::from_u16(upstream_status).unwrap_or(StatusCode::BAD_GATEWAY);

    let mut response = Response::builder()
        .status(status)
        .body(axum_body)
        .map_err(|e| ApiError::internal(format!("Failed to build response: {}", e)))?;

    // Copy response headers (skip hop-by-hop)
    for (key, value) in resp_headers {
        if is_hop_by_hop(&key) {
            continue;
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            response.headers_mut().insert(name, val);
        }
    }

    Ok(response)
}
