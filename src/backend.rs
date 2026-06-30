use anyhow::Result;
use dioxus::fullstack::{AsStatusCode, StatusCode};
use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::fullstack::Lazy;
#[cfg(feature = "server")]
use dioxus::server::axum;
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

const BASE62: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const URL_LENGTH: usize = 6;
const MAX_RETRIES: usize = 5;

#[cfg(feature = "server")]
fn encode(id: usize) -> String {
    let mut remaining = id;
    let mut digits = Vec::new();
    while remaining > 0 {
        let carry = remaining % BASE62.len();
        digits.push(carry);
        remaining /= BASE62.len();
    }

    while digits.len() < URL_LENGTH {
        digits.push(0);
    }
    digits.reverse();
    digits.iter().map(|&d| BASE62[d] as char).collect()
}

#[post("/api/url")]
pub async fn create_url(request: CreateUrlRequest) -> Result<String, CreateUrlError> {
    validate_url(&request.url)?;
    if let Some(alias) = &request.alias {
        validate_alias(alias)?;

        let inserted = sqlx::query_scalar::<_, String>(
            "INSERT INTO urls (hash, url, expiration) VALUES (?, ?, ?) RETURNING hash",
        )
        .bind(alias)
        .bind(request.url.as_str())
        .bind(request.expiration.as_deref())
        .fetch_one(&*DB)
        .await;

        match inserted {
            Ok(hash) => return Ok(format!("{}/{}", base_url().trim_end_matches('/'), hash)),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                return Err(CreateUrlError::AliasTaken);
            }
            Err(_) => return Err(CreateUrlError::Internal),
        }
    }

    for _ in 0..MAX_RETRIES {
        let id = fastrand::usize(0..BASE62.len().pow(URL_LENGTH as u32));
        let hash: String = encode(id);

        let inserted = sqlx::query_scalar::<_, String>(
            "INSERT INTO urls (hash, url, expiration) VALUES (?, ?, ?) RETURNING hash",
        )
        .bind(hash)
        .bind(request.url.as_str())
        .bind(request.expiration.as_deref())
        .fetch_one(&*DB)
        .await;

        match inserted {
            Ok(hash) => return Ok(format!("{}/{}", base_url().trim_end_matches('/'), hash)),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                continue;
            }
            Err(_) => return Err(CreateUrlError::Internal),
        }
    }

    return Err(CreateUrlError::Internal);
}

#[cfg(feature = "server")]
pub async fn redirect(
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let found = sqlx::query_scalar::<_, String>(
        "UPDATE urls SET redirects = redirects + 1
         WHERE hash = ? AND (expiration IS NULL OR expiration >= date('now'))
         RETURNING url",
    )
    .bind(&hash)
    .fetch_optional(&*DB)
    .await;

    match found {
        Ok(Some(url)) => axum::response::Redirect::temporary(&url).into_response(), // 307
        Ok(None) => axum::http::StatusCode::NOT_FOUND.into_response(),
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub total_links: i64,
    pub total_redirects: i64,
}

#[get("/api/stats")]
pub async fn get_stats() -> Result<Stats, ServerFnError> {
    let (total_links, total_redirects) =
        sqlx::query_as::<_, (i64, i64)>("SELECT COUNT(*), COALESCE(SUM(redirects), 0) FROM urls")
            .fetch_one(&*DB)
            .await
            .map_err(ServerFnError::new)?;

    Ok(Stats {
        total_links,
        total_redirects,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentLink {
    pub short: String,
    pub url: String,
    pub clicks: i64,
}

#[get("/api/recent")]
pub async fn get_recent_links() -> Result<Vec<RecentLink>, ServerFnError> {
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT hash, url, redirects FROM urls ORDER BY created_at DESC, id DESC LIMIT 25",
    )
    .fetch_all(&*DB)
    .await
    .map_err(ServerFnError::new)?;

    Ok(rows
        .into_iter()
        .map(|(hash, url, clicks)| RecentLink {
            short: format!("{}/{}", base_url().trim_end_matches('/'), hash),
            url,
            clicks,
        })
        .collect())
}
