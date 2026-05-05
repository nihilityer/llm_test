// Database queries for the "submissions" table.

use crate::models::*;
use tracing::debug;
use worker::*;

const SUBMISSION_COLS: &str =
    "id, submitter_type, user_id, ip_hash, website_id, model_id, \
     test_suite_version, api_style, endpoint_hash, total_score, \
     dimension_scores, test_results, created_at";

#[allow(dead_code)]
pub async fn find_oauth_submission(
    db: &D1Database,
    user_id: &str,
    website_id: &str,
    model_id: &str,
) -> Result<Option<SubmissionRow>> {
    db.prepare(
        "SELECT id, submitter_type, user_id, ip_hash, website_id, model_id, test_suite_version, \
         api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
         FROM submissions WHERE submitter_type = 'oauth' AND user_id = ? AND website_id = ? AND model_id = ?",
    )
    .bind(&[
        user_id.to_string().into(),
        website_id.to_string().into(),
        model_id.to_string().into(),
    ])?
    .first::<SubmissionRow>(None)
    .await
}

#[allow(dead_code)]
pub async fn find_anonymous_submission(
    db: &D1Database,
    ip_hash: &str,
    website_id: &str,
    model_id: &str,
) -> Result<Option<SubmissionRow>> {
    db.prepare(
        "SELECT id, submitter_type, user_id, ip_hash, website_id, model_id, test_suite_version, \
         api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
         FROM submissions WHERE submitter_type = 'anonymous' AND ip_hash = ? AND website_id = ? AND model_id = ?",
    )
    .bind(&[
        ip_hash.to_string().into(),
        website_id.to_string().into(),
        model_id.to_string().into(),
    ])?
    .first::<SubmissionRow>(None)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_submission(
    db: &D1Database,
    id: &str,
    submitter_type: &str,
    user_id: &Option<String>,
    ip_hash: &Option<String>,
    website_id: &str,
    model_id: &str,
    test_suite_version: &str,
    api_style: &str,
    endpoint_hash: &str,
    total_score: f64,
    dimension_scores: &str,
    test_results: &str,
) -> Result<()> {
    debug!(
        "Upserting submission: type={}, website={}, model={}, score={}",
        submitter_type, website_id, model_id, total_score
    );

    // Delete existing record, then insert
    if submitter_type == "oauth" {
        let uid = user_id.as_deref().unwrap_or("");
        db.prepare(
            "DELETE FROM submissions WHERE submitter_type = 'oauth' AND user_id = ? AND website_id = ? AND model_id = ?",
        )
        .bind(&[
            uid.into(),
            website_id.to_string().into(),
            model_id.to_string().into(),
        ])?
        .run()
        .await?;
    } else {
        let ih = ip_hash.as_deref().unwrap_or("");
        db.prepare(
            "DELETE FROM submissions WHERE submitter_type = 'anonymous' AND ip_hash = ? AND website_id = ? AND model_id = ?",
        )
        .bind(&[
            ih.into(),
            website_id.to_string().into(),
            model_id.to_string().into(),
        ])?
        .run()
        .await?;
    }

    let uid_val: String = user_id.clone().unwrap_or_default();
    let ip_val: String = ip_hash.clone().unwrap_or_default();

    db.prepare(
        "INSERT INTO submissions (id, submitter_type, user_id, ip_hash, website_id, model_id, \
         test_suite_version, api_style, endpoint_hash, total_score, \
         dimension_scores, test_results) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&[
        id.to_string().into(),
        submitter_type.to_string().into(),
        uid_val.into(),
        ip_val.into(),
        website_id.to_string().into(),
        model_id.to_string().into(),
        test_suite_version.to_string().into(),
        api_style.to_string().into(),
        endpoint_hash.to_string().into(),
        total_score.into(),
        dimension_scores.to_string().into(),
        test_results.to_string().into(),
    ])?
    .run()
    .await?;

    Ok(())
}

pub async fn get_all_submissions_for_rankings(
    db: &D1Database,
    api_style: Option<&str>,
) -> Result<Vec<SubmissionRow>> {
    if let Some(style) = api_style {
        db.prepare(format!(
            "SELECT {} FROM submissions WHERE api_style = ? ORDER BY created_at DESC",
            SUBMISSION_COLS
        ))
        .bind(&[style.to_string().into()])?
        .all()
        .await?
        .results()
    } else {
        db.prepare(format!(
            "SELECT {} FROM submissions ORDER BY created_at DESC",
            SUBMISSION_COLS
        ))
        .all()
        .await?
        .results()
    }
}

pub async fn get_submissions_by_website(
    db: &D1Database,
    website_id: &str,
) -> Result<Vec<SubmissionRow>> {
    db.prepare(format!(
        "SELECT {} FROM submissions WHERE website_id = ? ORDER BY created_at DESC",
        SUBMISSION_COLS
    ))
    .bind(&[website_id.to_string().into()])?
    .all()
    .await?
    .results()
}

pub async fn get_submissions_by_model(
    db: &D1Database,
    model_id: &str,
) -> Result<Vec<SubmissionRow>> {
    db.prepare(format!(
        "SELECT {} FROM submissions WHERE model_id = ? ORDER BY created_at DESC",
        SUBMISSION_COLS
    ))
    .bind(&[model_id.to_string().into()])?
    .all()
    .await?
    .results()
}
