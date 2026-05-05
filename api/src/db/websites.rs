// Database queries for the "websites" table.

use crate::models::*;
use worker::*;

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

pub async fn get_all_websites(db: &D1Database) -> Result<Vec<WebsiteRow>> {
    db.prepare("SELECT id, name, domains FROM websites ORDER BY name")
        .all()
        .await?
        .results()
}
