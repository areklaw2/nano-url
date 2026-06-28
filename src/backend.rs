use anyhow::Result;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
#[cfg(feature = "server")]
use std::{str::FromStr, sync::OnceLock};

#[cfg(feature = "server")]
static DB: OnceLock<SqlitePool> = OnceLock::new();

#[cfg(feature = "server")]
pub async fn get_db() -> &'static SqlitePool {
    if let Some(pool) = DB.get() {
        return pool;
    }
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite://nano_url.db")
            .expect("Invalid database URL")
            .create_if_missing(true),
    )
    .await
    .expect("Failed to open database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    DB.get_or_init(|| pool)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub alias: Option<String>,
    pub expiration: Option<String>,
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
pub async fn create_url(request: CreateUrlRequest) -> Result<(), ServerFnError> {
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
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ServerFnError::new("That nano url is already taken, try another one")
            }
            _ => ServerFnError::new("Something went wrong creating your link, please try again"),
        })?;
    Ok(())
}
