use anyhow::Result;
use dioxus::fullstack::{AsStatusCode, StatusCode};
use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::fullstack::Lazy;
#[cfg(feature = "server")]
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
#[cfg(feature = "server")]
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "server")]
static DB: Lazy<SqlitePool> = Lazy::new(|| async {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite://nano_url.db")?.create_if_missing(true),
    )
    .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    dioxus::Ok(pool)
});

#[cfg(feature = "server")]
fn base_url() -> &'static str {
    static BASE_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BASE_URL.get_or_init(|| {
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into())
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub alias: Option<String>,
    pub expiration: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Error)]
pub enum CreateUrlError {
    #[error("{0}")]
    BadRequest(String),
    #[error("That nano url is already taken, try another one")]
    AliasTaken,
    #[error("Something went wrong creating your link, please try again")]
    Internal,
}

impl AsStatusCode for CreateUrlError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            CreateUrlError::BadRequest(_) => StatusCode::BAD_REQUEST,
            CreateUrlError::AliasTaken => StatusCode::CONFLICT,
            CreateUrlError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ServerFnError> for CreateUrlError {
    fn from(_: ServerFnError) -> Self {
        CreateUrlError::Internal
    }
}

fn validate_url(url: &str) -> Result<(), CreateUrlError> {
    if url.trim().is_empty() {
        return Err(CreateUrlError::BadRequest(
            "url is empty or whitespace".into(),
        ));
    }
    if url.contains(char::is_whitespace) {
        return Err(CreateUrlError::BadRequest(
            "url must not contain whitespace".into(),
        ));
    }
    Ok(())
}

fn validate_alias(alias: &str) -> Result<(), CreateUrlError> {
    if alias.len() < 5 {
        return Err(CreateUrlError::BadRequest(
            "alias must be at least 5 characters".into(),
        ));
    }
    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CreateUrlError::BadRequest(
            "alias can only contain letters, numbers, - and _".into(),
        ));
    }
    Ok(())
}

#[post("/api/url")]
pub async fn create_url(request: CreateUrlRequest) -> Result<String, CreateUrlError> {
    validate_url(&request.url)?;
    if let Some(alias) = &request.alias {
        validate_alias(alias)?;
    }

    let hash = "test2";
    let inserted = sqlx::query_scalar::<_, String>(
        "INSERT INTO urls (hash, url, expiration) VALUES (?, ?, ?) RETURNING hash",
    )
    .bind(hash)
    .bind(request.url)
    .bind(request.expiration)
    .fetch_one(&*DB)
    .await;

    match inserted {
        Ok(hash) => Ok(format!("{}/{}", base_url().trim_end_matches('/'), hash)),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(CreateUrlError::AliasTaken),
        Err(_) => Err(CreateUrlError::Internal),
    }
}
