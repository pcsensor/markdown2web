use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use crate::{app::AppState, config::AppConfig, error::AppResult, store::sqlite::SessionRecord};

pub const SESSION_COOKIE: &str = "m2w_session";
pub const USER_SESSION_COOKIE: &str = "m2w_user_session";

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub username: String,
    pub csrf_token: String,
    pub is_admin: bool,
}

pub fn build_session_cookie(token: String, config: &AppConfig) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .secure(config.secure_cookies)
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
    Ok(current_admin_session(jar, state)?.map(|session| session.username))
}

pub fn current_admin_session(jar: &CookieJar, state: &AppState) -> AppResult<Option<AuthSession>> {
    match session_token(jar) {
        Some(token) => Ok(state
            .db
            .session_user(&token)?
            .map(|session| auth_session(session, true))),
        None => Ok(None),
    }
}

pub fn build_user_session_cookie(token: String, config: &AppConfig) -> Cookie<'static> {
    Cookie::build((USER_SESSION_COOKIE, token))
        .http_only(true)
        .secure(config.secure_cookies)
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
    Ok(current_public_session(jar, state)?.map(|session| session.username))
}

pub fn current_public_session(jar: &CookieJar, state: &AppState) -> AppResult<Option<AuthSession>> {
    match user_session_token(jar) {
        Some(token) => Ok(state
            .db
            .public_session_user(&token)?
            .map(|session| auth_session(session, false))),
        None => Ok(None),
    }
}

/// Unified viewer: tries public user first, then admin user.
/// Returns (username, is_admin) tuple.
pub fn current_viewer(jar: &CookieJar, state: &AppState) -> AppResult<Option<(String, bool)>> {
    Ok(current_viewer_session(jar, state)?.map(|session| (session.username, session.is_admin)))
}

pub fn current_viewer_session(jar: &CookieJar, state: &AppState) -> AppResult<Option<AuthSession>> {
    if let Some(session) = current_public_session(jar, state)? {
        return Ok(Some(session));
    }
    current_admin_session(jar, state)
}

fn auth_session(record: SessionRecord, is_admin: bool) -> AuthSession {
    AuthSession {
        username: record.username,
        csrf_token: record.csrf_token,
        is_admin,
    }
}
