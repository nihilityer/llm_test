use crate::db;
use crate::error::ApiError;
use crate::models::*;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Deserialize, Default)]
pub struct RankingsQuery {
    pub website_id: Option<String>,
    pub model_id: Option<String>,
    pub ranking_type: Option<String>,
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
    info!(
        "Rankings request: type={:?}, style={:?}",
        query.ranking_type, query.style
    );
    // Website detail mode
    if let Some(ref website_id) = query.website_id {
        return website_detail(&state, website_id)
            .await
            .map(|r| r.into_response());
    }

    // Model detail mode
    if let Some(ref model_id) = query.model_id {
        return model_detail(&state, model_id)
            .await
            .map(|r| r.into_response());
    }

    // Type-based dispatch
    match query.ranking_type.as_deref() {
        Some("model") => model_rankings(&state, query).await.map(|r| r.into_response()),
        _ => website_rankings(&state, query).await.map(|r| r.into_response()),
    }
}

// ---------- Website rankings ----------

async fn website_rankings(
    state: &AppState,
    query: RankingsQuery,
) -> Result<Json<RankingsResponse<RankingEntry>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let submissions = db::get_all_submissions_for_rankings(
        &state.database,
        query.style.as_deref(),
    )
    .await?;
    let websites = db::get_all_websites(&state.database).await?;
    let website_models = db::get_all_website_models(&state.database).await?;

    let model_weight_map: HashMap<(String, String), f64> = website_models
        .iter()
        .map(|wm| ((wm.website_id.clone(), wm.model_id.clone()), wm.model_weight))
        .collect();

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
                let mw = model_weight_map
                    .get(&(s.website_id.clone(), s.model_id.clone()))
                    .copied()
                    .unwrap_or(1.0);
                s.total_score * vw * sw * mw
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
    }))
}

// ---------- Model rankings ----------

async fn model_rankings(
    state: &AppState,
    query: RankingsQuery,
) -> Result<Json<RankingsResponse<ModelRankingEntry>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let submissions = db::get_all_submissions_for_rankings(
        &state.database,
        query.style.as_deref(),
    )
    .await?;
    let models = db::get_all_models(&state.database).await?;
    let website_models = db::get_all_website_models(&state.database).await?;

    let website_weight_map: HashMap<(String, String), f64> = website_models
        .iter()
        .map(|wm| ((wm.website_id.clone(), wm.model_id.clone()), wm.website_weight))
        .collect();

    let mut model_subs: HashMap<String, Vec<&SubmissionRow>> = HashMap::new();
    for sub in &submissions {
        model_subs
            .entry(sub.model_id.clone())
            .or_default()
            .push(sub);
    }

    let weight_config = state.get_weight_config();

    let mut entries: Vec<ModelRankingEntry> = Vec::new();
    for model in &models {
        let subs = model_subs.get(&model.id);

        let sub_count = subs.map(|s| s.len()).unwrap_or(0) as u32;
        if sub_count == 0 {
            continue;
        }

        let subs = subs.unwrap();

        let mut website_ids: Vec<&str> = subs.iter().map(|s| s.website_id.as_str()).collect();
        website_ids.sort_unstable();
        website_ids.dedup();
        let website_count = website_ids.len() as u32;

        if let Some(ref search) = query.search {
            let search_lower = search.to_lowercase();
            let name_match = model.name.to_lowercase().contains(&search_lower);
            let alias_match = model.aliases.to_lowercase().contains(&search_lower);
            if !name_match && !alias_match {
                continue;
            }
        }

        let weighted_scores: Vec<f64> = subs
            .iter()
            .map(|s| {
                let vw = weight_config.get_version_weight(&s.test_suite_version);
                let sw = weight_config.get_submitter_weight(&s.submitter_type);
                let ww = website_weight_map
                    .get(&(s.website_id.clone(), s.model_id.clone()))
                    .copied()
                    .unwrap_or(1.0);
                s.total_score * vw * sw * ww
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

        let aliases: Vec<String> =
            serde_json::from_str(&model.aliases).unwrap_or_default();

        entries.push(ModelRankingEntry {
            rank: 0,
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            aliases,
            avg_score: (avg_score * 100.0).round() / 100.0,
            submission_count: sub_count,
            max_score: (max_score * 100.0).round() / 100.0,
            min_score: (min_score * 100.0).round() / 100.0,
            last_tested_at: last_tested,
            website_count,
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

    let paged: Vec<ModelRankingEntry> = entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(Json(RankingsResponse {
        rankings: paged,
        total,
    }))
}

// ---------- Website detail ----------

async fn website_detail(
    state: &AppState,
    website_id: &str,
) -> Result<Json<WebsiteDetailResponse>, ApiError> {
    let website = db::find_website_by_id(&state.database, website_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Website not found".into()))?;

    let submissions = db::get_submissions_by_website(&state.database, website_id).await?;
    let models = db::get_all_models(&state.database).await?;
    let model_map: HashMap<String, &ModelRow> = models.iter().map(|m| (m.id.clone(), m)).collect();

    let user_ids: Vec<String> = submissions
        .iter()
        .filter(|s| s.submitter_type == "oauth")
        .filter_map(|s| s.user_id.clone())
        .collect();

    let users = db::get_all_oauth_users(&state.database, &user_ids).await?;

    let weight_config = state.get_weight_config();
    let website_models = db::get_all_website_models(&state.database).await?;
    let model_weight_map: HashMap<(String, String), f64> = website_models
        .iter()
        .map(|wm| ((wm.website_id.clone(), wm.model_id.clone()), wm.model_weight))
        .collect();

    let sub_count = submissions.len() as u32;

    let weighted_scores: Vec<f64> = submissions
        .iter()
        .map(|s| {
            let vw = weight_config.get_version_weight(&s.test_suite_version);
            let sw = weight_config.get_submitter_weight(&s.submitter_type);
            let mw = model_weight_map
                .get(&(s.website_id.clone(), s.model_id.clone()))
                .copied()
                .unwrap_or(1.0);
            s.total_score * vw * sw * mw
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

            let model_name = model_map
                .get(&s.model_id)
                .map(|m| m.name.clone())
                .unwrap_or_default();

            SubmissionDetail {
                id: s.id.clone(),
                submitter_type: s.submitter_type.clone(),
                submitter_name: name,
                submitter_avatar: avatar,
                model_id: s.model_id.clone(),
                model_name,
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

// ---------- Model detail ----------

async fn model_detail(
    state: &AppState,
    model_id: &str,
) -> Result<Json<ModelDetailResponse>, ApiError> {
    let model = db::find_model_by_id(&state.database, model_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Model not found".into()))?;

    let submissions = db::get_submissions_by_model(&state.database, model_id).await?;

    let user_ids: Vec<String> = submissions
        .iter()
        .filter(|s| s.submitter_type == "oauth")
        .filter_map(|s| s.user_id.clone())
        .collect();

    let users = db::get_all_oauth_users(&state.database, &user_ids).await?;

    let weight_config = state.get_weight_config();
    let website_models = db::get_all_website_models(&state.database).await?;
    let website_weight_map: HashMap<(String, String), f64> = website_models
        .iter()
        .map(|wm| ((wm.website_id.clone(), wm.model_id.clone()), wm.website_weight))
        .collect();

    let sub_count = submissions.len() as u32;

    let weighted_scores: Vec<f64> = submissions
        .iter()
        .map(|s| {
            let vw = weight_config.get_version_weight(&s.test_suite_version);
            let sw = weight_config.get_submitter_weight(&s.submitter_type);
            let ww = website_weight_map
                .get(&(s.website_id.clone(), s.model_id.clone()))
                .copied()
                .unwrap_or(1.0);
            s.total_score * vw * sw * ww
        })
        .collect();
    let total: f64 = weighted_scores.iter().sum();
    let avg_score = if !weighted_scores.is_empty() {
        total / weighted_scores.len() as f64
    } else {
        0.0
    };

    let mut distinct_website_ids: Vec<&str> =
        submissions.iter().map(|s| s.website_id.as_str()).collect();
    distinct_website_ids.sort_unstable();
    distinct_website_ids.dedup();
    let website_count = distinct_website_ids.len() as u32;

    let aliases: Vec<String> = serde_json::from_str(&model.aliases).unwrap_or_default();

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
                model_id: s.model_id.clone(),
                model_name: model.name.clone(),
                total_score: s.total_score,
                dimension_scores: serde_json::from_str(&s.dimension_scores).unwrap_or_default(),
                test_suite_version: s.test_suite_version.clone(),
                api_style: s.api_style.clone(),
                created_at: s.created_at.clone(),
            }
        })
        .collect();

    Ok(Json(ModelDetailResponse {
        model: ModelSummary {
            id: model.id,
            name: model.name,
            aliases,
            avg_score: (avg_score * 100.0).round() / 100.0,
            submission_count: sub_count,
            website_count,
        },
        submissions: submission_details,
    }))
}
