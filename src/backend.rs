use anyhow::Result;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use sqlx::SqlitePool;
#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
static DB: OnceLock<SqlitePool> = OnceLock::new();

#[cfg(feature = "server")]
pub async fn get_db() -> &'static SqlitePool {
    if let Some(pool) = DB.get() {
        return pool;
    }
    let pool = SqlitePool::connect("sqlite://nano_url.db")
        .await
        .expect("Failed to open database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    DB.get_or_init(|| pool)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateUrl {
    url: String,
    alias: Option<String>,
    expiration: Option<String>,
}

fn validate_url(url: &str) -> Result<(), ServerFnError> {
    if url.trim().is_empty() {
        return Err(ServerFnError::new("url is empty or whitespace"));
    }
    if url.contains(char::is_whitespace) {
        return Err(ServerFnError::new("url must not contain whitespace"));
    }
    Ok(())
}

fn validate_alias(alias: &str) -> Result<(), ServerFnError> {
    if alias.len() < 5 {
        return Err(ServerFnError::new("alias must be at least 5 characters"));
    }
    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ServerFnError::new(
            "alias can only contain letters, numbers, - and _",
        ));
    }
    Ok(())
}

#[post("/api/url")]
pub async fn create_url(request: CreateUrl) -> Result<(), ServerFnError> {
    validate_url(&request.url)?;
    if let Some(alias) = &request.alias {
        validate_alias(alias)?;
    }

    let db = get_db().await;
    sqlx::query("INSERT INTO urls (hash, url, expiration) VALUES (?, ?, ?)")
        .bind("test")
        .bind(request.url)
        .bind(request.expiration)
        .execute(db)
        .await
        .map_err(ServerFnError::new)?;
    Ok(())
}
