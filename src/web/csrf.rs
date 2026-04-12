use axum::http::HeaderMap;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rand::{Rng, distributions::Alphanumeric};

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
};

pub const PRE_AUTH_CSRF_COOKIE: &str = "m2w_csrf";
pub const CSRF_FORM_FIELD: &str = "_csrf";
pub const CSRF_HEADER: &str = "x-csrf-token";

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn build_pre_auth_cookie(token: String, config: &AppConfig) -> Cookie<'static> {
    Cookie::build((PRE_AUTH_CSRF_COOKIE, token))
        .http_only(true)
        .secure(config.secure_cookies)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub fn clear_pre_auth_cookie() -> Cookie<'static> {
    Cookie::build((PRE_AUTH_CSRF_COOKIE, "")).path("/").build()
}

pub fn verify_pre_auth(jar: &CookieJar, submitted: &str) -> AppResult<()> {
    let expected = jar
        .get(PRE_AUTH_CSRF_COOKIE)
        .map(|cookie| cookie.value())
        .unwrap_or_default();
    verify_pair(expected, submitted)
}

pub fn verify_form(expected: &str, submitted: &str) -> AppResult<()> {
    verify_pair(expected, submitted)
}

pub fn verify_header(headers: &HeaderMap, expected: &str) -> AppResult<()> {
    let submitted = headers
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    verify_pair(expected, submitted)
}

fn verify_pair(expected: &str, submitted: &str) -> AppResult<()> {
    if expected.is_empty() || submitted.is_empty() || !constant_time_eq(expected, submitted) {
        return Err(AppError::BadRequest("invalid csrf token".into()));
    }
    Ok(())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();

    for index in 0..max_len {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        diff |= (a ^ b) as usize;
    }

    diff == 0
}
