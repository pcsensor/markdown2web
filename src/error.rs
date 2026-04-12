use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("too many requests: {0}")]
    RateLimited(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn internal<E: std::fmt::Display>(error: E) -> Self {
        Self::Internal(error.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            AppError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        let body = format!(
            "<html><head><title>{}</title></head><body><h1>{}</h1><p>{}</p></body></html>",
            title, title, self
        );
        (status, Html(body)).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::internal(value)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::internal(value)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::internal(value)
    }
}

impl From<notify::Error> for AppError {
    fn from(value: notify::Error) -> Self {
        Self::internal(value)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::internal(value)
    }
}
