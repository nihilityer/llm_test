// Database queries for the "models" table.
// NOTE: This module ("crate::db::models") contains database queries.
// It is distinct from "crate::models" which defines type/struct declarations.

use crate::models::*;
use worker::*;

pub async fn find_model_by_name_or_alias(
    db: &D1Database,
    name: &str,
) -> Result<Option<ModelRow>> {
    let pattern = format!("%\"{}\"%", name);
    db.prepare("SELECT id, name, aliases FROM models WHERE name = ? OR aliases LIKE ? LIMIT 1")
        .bind(&[name.to_string().into(), pattern.into()])?
        .first::<ModelRow>(None)
        .await
}

pub async fn create_model(db: &D1Database, id: &str, name: &str) -> Result<ModelRow> {
    db.prepare("INSERT INTO models (id, name, aliases) VALUES (?, ?, '[]')")
        .bind(&[id.to_string().into(), name.to_string().into()])?
        .run()
        .await?;
    Ok(ModelRow {
        id: id.to_string(),
        name: name.to_string(),
        aliases: "[]".to_string(),
    })
}

pub async fn find_model_by_id(db: &D1Database, id: &str) -> Result<Option<ModelRow>> {
    db.prepare("SELECT id, name, aliases FROM models WHERE id = ?")
        .bind(&[id.to_string().into()])?
        .first::<ModelRow>(None)
        .await
}

pub async fn get_all_models(db: &D1Database) -> Result<Vec<ModelRow>> {
    db.prepare("SELECT id, name, aliases FROM models ORDER BY name")
        .all()
        .await?
        .results()
}
