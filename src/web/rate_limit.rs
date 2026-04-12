use axum::http::HeaderMap;

use crate::{app::AppState, error::AppResult};

pub const LOGIN_LIMIT: i64 = 5;
pub const LOGIN_WINDOW_SECS: i64 = 15 * 60;
pub const PUBLIC_AUTH_LIMIT: i64 = 10;
pub const PUBLIC_AUTH_WINDOW_SECS: i64 = 15 * 60;
pub const API_WRITE_LIMIT: i64 = 60;
pub const API_WRITE_WINDOW_SECS: i64 = 60;

pub fn check(
    state: &AppState,
    namespace: &str,
    subject: &str,
    headers: &HeaderMap,
    limit: i64,
    window_secs: i64,
) -> AppResult<()> {
    let key = format!(
        "{}:{}:{}",
        namespace,
        normalize_subject(subject),
        client_fingerprint(headers)
    );
    state.db.check_rate_limit(&key, limit, window_secs)
}

fn normalize_subject(subject: &str) -> String {
    subject
        .trim()
        .chars()
        .take(96)
        .map(|ch| match ch {
            ':' | '\n' | '\r' | '\t' => '_',
            _ => ch,
        })
        .collect()
}

fn client_fingerprint(headers: &HeaderMap) -> String {
    let value = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
        })
        .unwrap_or("unknown");

    normalize_subject(value)
}
