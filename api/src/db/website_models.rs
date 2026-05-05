// Database queries for the "website_models" join table.

use crate::models::*;
use worker::*;

pub async fn find_website_model(
    db: &D1Database,
    website_id: &str,
    model_id: &str,
) -> Result<Option<WebsiteModelRow>> {
    db.prepare(
        "SELECT website_id, model_id, model_weight, website_weight \
         FROM website_models WHERE website_id = ? AND model_id = ?",
    )
    .bind(&[website_id.to_string().into(), model_id.to_string().into()])?
    .first::<WebsiteModelRow>(None)
    .await
}

pub async fn create_website_model(
    db: &D1Database,
    website_id: &str,
    model_id: &str,
) -> Result<WebsiteModelRow> {
    db.prepare(
        "INSERT INTO website_models (website_id, model_id, model_weight, website_weight) \
         VALUES (?, ?, 1.0, 1.0)",
    )
    .bind(&[website_id.to_string().into(), model_id.to_string().into()])?
    .run()
    .await?;
    Ok(WebsiteModelRow {
        website_id: website_id.to_string(),
        model_id: model_id.to_string(),
        model_weight: 1.0,
        website_weight: 1.0,
    })
}

pub async fn get_all_website_models(db: &D1Database) -> Result<Vec<WebsiteModelRow>> {
    db.prepare("SELECT website_id, model_id, model_weight, website_weight FROM website_models")
        .all()
        .await?
        .results()
}
