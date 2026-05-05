// Rate limiting queries.

use worker::*;

#[derive(serde::Deserialize)]
struct CountResult {
    cnt: Option<u32>,
}

pub async fn count_anonymous_submissions_by_ip(
    db: &D1Database,
    ip_hash: &str,
    within_hours: u32,
) -> Result<u32> {
    let result = db
        .prepare(
            "SELECT COUNT(*) as cnt FROM submissions \
             WHERE submitter_type = 'anonymous' AND ip_hash = ? \
             AND created_at > datetime('now', ?)",
        )
        .bind(&[
            ip_hash.to_string().into(),
            format!("-{} hours", within_hours).into(),
        ])?
        .first::<CountResult>(None)
        .await?;

    Ok(result.and_then(|r| r.cnt).unwrap_or(0))
}

pub async fn count_anonymous_submissions_by_ip_and_website(
    db: &D1Database,
    ip_hash: &str,
    website_id: &str,
    within_hours: u32,
) -> Result<u32> {
    let result = db
        .prepare(
            "SELECT COUNT(*) as cnt FROM submissions \
             WHERE submitter_type = 'anonymous' AND ip_hash = ? AND website_id = ? \
             AND created_at > datetime('now', ?)",
        )
        .bind(&[
            ip_hash.to_string().into(),
            website_id.to_string().into(),
            format!("-{} hours", within_hours).into(),
        ])?
        .first::<CountResult>(None)
        .await?;

    Ok(result.and_then(|r| r.cnt).unwrap_or(0))
}
