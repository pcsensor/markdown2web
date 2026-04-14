use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use crate::{app::AppState, error::AppResult};

pub const SESSION_COOKIE: &str = "m2w_session";
pub const USER_SESSION_COOKIE: &str = "m2w_user_session";

pub fn build_session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, "")).path("/").build()
}

pub fn session_token(jar: &CookieJar) -> Option<String> {
    jar.get(SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string())
}

pub fn current_user(jar: &CookieJar, state: &AppState) -> AppResult<Option<String>> {
    match session_token(jar) {
        Some(token) => state.db.session_user(&token),
        None => Ok(None),
    }
}

pub fn build_user_session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((USER_SESSION_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub fn clear_user_session_cookie() -> Cookie<'static> {
    Cookie::build((USER_SESSION_COOKIE, "")).path("/").build()
}

pub fn user_session_token(jar: &CookieJar) -> Option<String> {
    jar.get(USER_SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string())
}

pub fn current_public_user(jar: &CookieJar, state: &AppState) -> AppResult<Option<String>> {
    match user_session_token(jar) {
        Some(token) => state.db.public_session_user(&token),
        None => Ok(None),
    }
}

/// Unified viewer: tries public user first, then admin user.
/// Returns (username, is_admin) tuple.
pub fn current_viewer(jar: &CookieJar, state: &AppState) -> AppResult<Option<(String, bool)>> {
    if let Some(username) = current_public_user(jar, state)? {
        return Ok(Some((username, false)));
    }
    if let Some(username) = current_user(jar, state)? {
        return Ok(Some((username, true)));
    }
    Ok(None)
}
