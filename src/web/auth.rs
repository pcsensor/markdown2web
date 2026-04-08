use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use crate::{app::AppState, error::AppResult};

pub const SESSION_COOKIE: &str = "m2w_session";

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
