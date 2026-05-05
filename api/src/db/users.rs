// Database queries for the "users" table.

use crate::models::*;
use std::collections::HashMap;
use worker::*;

pub async fn find_user_by_github_id(db: &D1Database, github_id: i32) -> Result<Option<UserRow>> {
    db.prepare("SELECT id, github_id, login, avatar_url FROM users WHERE github_id = ?")
        .bind(&[github_id.into()])?
        .first::<UserRow>(None)
        .await
}

pub async fn create_user(
    db: &D1Database,
    id: &str,
    github_id: i32,
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
