use crate::models::*;
use std::collections::HashMap;
use worker::*;

// ---------- Users ----------

pub async fn find_user_by_github_id(db: &D1Database, github_id: i64) -> Result<Option<UserRow>> {
    db.prepare("SELECT id, github_id, login, avatar_url FROM users WHERE github_id = ?")
        .bind(&[github_id.into()])?
        .first::<UserRow>(None)
        .await
}

pub async fn create_user(
    db: &D1Database,
    id: &str,
    github_id: i64,
    login: &str,
    avatar_url: &Option<String>,
) -> Result<()> {
    db.prepare("INSERT INTO users (id, github_id, login, avatar_url) VALUES (?, ?, ?, ?)")
        .bind(&[
            id.to_string().into(),
            github_id.into(),
            login.to_string().into(),
            avatar_url.clone().unwrap_or_default().into(),
        ])?
        .run()
        .await?;
    Ok(())
}

// ---------- Websites ----------

pub async fn find_website_by_id(db: &D1Database, id: &str) -> Result<Option<WebsiteRow>> {
    db.prepare("SELECT id, name, domains FROM websites WHERE id = ?")
        .bind(&[id.to_string().into()])?
        .first::<WebsiteRow>(None)
        .await
}

pub async fn find_website_by_domain(db: &D1Database, domain: &str) -> Result<Option<WebsiteRow>> {
    let pattern = format!("%\"{}\"%", domain);
    db.prepare("SELECT id, name, domains FROM websites WHERE domains LIKE ? LIMIT 1")
        .bind(&[pattern.into()])?
        .first::<WebsiteRow>(None)
        .await
}

pub async fn create_website(
    db: &D1Database,
    id: &str,
    name: &str,
    domains: &[String],
) -> Result<WebsiteRow> {
    let domains_json = serde_json::to_string(domains)?;
    db.prepare("INSERT INTO websites (id, name, domains) VALUES (?, ?, ?)")
        .bind(&[
            id.to_string().into(),
            name.to_string().into(),
            domains_json.clone().into(),
        ])?
        .run()
        .await?;
    Ok(WebsiteRow {
        id: id.to_string(),
        name: name.to_string(),
        domains: domains_json,
    })
}

// ---------- Submissions ----------

#[allow(dead_code)]
pub async fn find_oauth_submission(
    db: &D1Database,
    user_id: &str,
    website_id: &str,
) -> Result<Option<SubmissionRow>> {
    db.prepare(
        "SELECT id, submitter_type, user_id, ip_hash, website_id, test_suite_version, \
         api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
         FROM submissions WHERE submitter_type = 'oauth' AND user_id = ? AND website_id = ?",
    )
    .bind(&[user_id.to_string().into(), website_id.to_string().into()])?
    .first::<SubmissionRow>(None)
    .await
}

#[allow(dead_code)]
pub async fn find_anonymous_submission(
    db: &D1Database,
    ip_hash: &str,
    website_id: &str,
) -> Result<Option<SubmissionRow>> {
    db.prepare(
        "SELECT id, submitter_type, user_id, ip_hash, website_id, test_suite_version, \
         api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
         FROM submissions WHERE submitter_type = 'anonymous' AND ip_hash = ? AND website_id = ?",
    )
    .bind(&[ip_hash.to_string().into(), website_id.to_string().into()])?
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
    test_suite_version: &str,
    api_style: &str,
    endpoint_hash: &str,
    total_score: f64,
    dimension_scores: &str,
    test_results: &str,
) -> Result<()> {
    // Delete existing record, then insert (simple upsert strategy)
    if submitter_type == "oauth" {
        let uid = user_id.as_deref().unwrap_or("");
        db.prepare(
            "DELETE FROM submissions WHERE submitter_type = 'oauth' AND user_id = ? AND website_id = ?",
        )
        .bind(&[uid.into(), website_id.to_string().into()])?
        .run()
        .await?;
    } else {
        let ih = ip_hash.as_deref().unwrap_or("");
        db.prepare(
            "DELETE FROM submissions WHERE submitter_type = 'anonymous' AND ip_hash = ? AND website_id = ?",
        )
        .bind(&[ih.into(), website_id.to_string().into()])?
        .run()
        .await?;
    }

    let uid_val: String = user_id.clone().unwrap_or_default();
    let ip_val: String = ip_hash.clone().unwrap_or_default();

    db.prepare(
        "INSERT INTO submissions (id, submitter_type, user_id, ip_hash, website_id, \
         test_suite_version, api_style, endpoint_hash, total_score, \
         dimension_scores, test_results) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&[
        id.to_string().into(),
        submitter_type.to_string().into(),
        uid_val.into(),
        ip_val.into(),
        website_id.to_string().into(),
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

// ---------- Rate limiting ----------

pub async fn count_anonymous_submissions_by_ip(
    db: &D1Database,
    ip_hash: &str,
    within_hours: u32,
) -> Result<u32> {
    #[derive(serde::Deserialize)]
    struct CountResult {
        cnt: Option<u32>,
    }

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
    #[derive(serde::Deserialize)]
    struct CountResult {
        cnt: Option<u32>,
    }

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

// ---------- Rankings ----------

pub async fn get_all_submissions_for_rankings(
    db: &D1Database,
    api_style: Option<&str>,
) -> Result<Vec<SubmissionRow>> {
    if let Some(style) = api_style {
        db.prepare(
            "SELECT id, submitter_type, user_id, ip_hash, website_id, test_suite_version, \
             api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
             FROM submissions WHERE api_style = ? ORDER BY created_at DESC",
        )
        .bind(&[style.to_string().into()])?
        .all()
        .await?
        .results()
    } else {
        db.prepare(
            "SELECT id, submitter_type, user_id, ip_hash, website_id, test_suite_version, \
             api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
             FROM submissions ORDER BY created_at DESC",
        )
        .all()
        .await?
        .results()
    }
}

pub async fn get_submissions_by_website(
    db: &D1Database,
    website_id: &str,
) -> Result<Vec<SubmissionRow>> {
    db.prepare(
        "SELECT id, submitter_type, user_id, ip_hash, website_id, test_suite_version, \
         api_style, endpoint_hash, total_score, dimension_scores, test_results, created_at \
         FROM submissions WHERE website_id = ? ORDER BY created_at DESC",
    )
    .bind(&[website_id.to_string().into()])?
    .all()
    .await?
    .results()
}

pub async fn get_all_websites(db: &D1Database) -> Result<Vec<WebsiteRow>> {
    db.prepare("SELECT id, name, domains FROM websites ORDER BY name")
        .all()
        .await?
        .results()
}

pub async fn get_all_oauth_users(
    db: &D1Database,
    user_ids: &[String],
) -> Result<HashMap<String, UserRow>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders: Vec<String> = (0..user_ids.len()).map(|i| format!("?{}", i + 1)).collect();
    let query = format!(
        "SELECT id, github_id, login, avatar_url FROM users WHERE id IN ({})",
        placeholders.join(",")
    );

    let mut stmt = db.prepare(&query);
    for id in user_ids {
        stmt = stmt.bind(&[id.clone().into()])?;
    }
    let rows: Vec<UserRow> = stmt.all().await?.results()?;

    let map: HashMap<String, UserRow> = rows.into_iter().map(|r| (r.id.clone(), r)).collect();
    Ok(map)
}
