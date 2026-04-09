use askama::Template;
use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    store::sqlite::{NewAnnotation, NoteAnnotation},
    web::auth,
};

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    site_name: String,
    current_user: Option<String>,
    next: String,
    login_username: String,
    register_username: String,
    login_error: Option<String>,
    register_error: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct AccountQuery {
    next: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    next: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    next: Option<String>,
}

#[derive(Deserialize)]
pub struct LogoutForm {
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnotationPayload {
    start_offset: usize,
    end_offset: usize,
    quote: String,
    color: Option<String>,
    comment: Option<String>,
    visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnnotationPayload {
    color: Option<String>,
    comment: Option<String>,
    visibility: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnnotationListResponse {
    annotations: Vec<NoteAnnotation>,
}

pub async fn account_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AccountQuery>,
) -> AppResult<Response> {
    render_account(
        &state,
        auth::current_public_user(&jar, &state)?,
        normalize_next(query.next.as_deref(), "/account"),
        String::new(),
        String::new(),
        None,
        None,
    )
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> AppResult<Response> {
    let next = normalize_next(form.next.as_deref(), "/account");
    if auth::current_public_user(&jar, &state)?.is_some() {
        return Ok(Redirect::to(&next).into_response());
    }

    let username = form.username.trim().to_string();
    if username.is_empty() {
        return render_account(
            &state,
            None,
            next,
            String::new(),
            username,
            None,
            Some("用户名不能为空。".into()),
        );
    }
    if form.password.trim().len() < 8 {
        return render_account(
            &state,
            None,
            next,
            String::new(),
            username,
            None,
            Some("密码至少需要 8 个字符。".into()),
        );
    }
    if !state.db.register_public_user(&username, &form.password)? {
        return render_account(
            &state,
            None,
            next,
            String::new(),
            username,
            None,
            Some("该用户名已被注册。".into()),
        );
    }
    let token = state.db.create_public_session(&username)?;
    Ok((
        jar.add(auth::build_user_session_cookie(token)),
        Redirect::to(&next),
    )
        .into_response())
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    let next = normalize_next(form.next.as_deref(), "/account");
    if auth::current_public_user(&jar, &state)?.is_some() {
        return Ok(Redirect::to(&next).into_response());
    }

    let username = form.username.trim().to_string();
    if !state.db.verify_public_user(&username, &form.password)? {
        return render_account(
            &state,
            None,
            next,
            username,
            String::new(),
            Some("用户名或密码错误。".into()),
            None,
        );
    }

    let token = state.db.create_public_session(&username)?;
    Ok((
        jar.add(auth::build_user_session_cookie(token)),
        Redirect::to(&next),
    )
        .into_response())
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LogoutForm>,
) -> AppResult<Response> {
    if let Some(token) = auth::user_session_token(&jar) {
        let _ = state.db.delete_public_session(&token);
    }
    Ok((
        jar.remove(auth::clear_user_session_cookie()),
        Redirect::to(&normalize_next(form.next.as_deref(), "/account")),
    )
        .into_response())
}

pub async fn list_annotations(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Json<AnnotationListResponse>> {
    ensure_published_note(&state, &slug).await?;
    let username = auth::current_public_user(&jar, &state)?;
    Ok(Json(AnnotationListResponse {
        annotations: state
            .db
            .list_visible_annotations(&slug, username.as_deref())?,
    }))
}

pub async fn create_annotation(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateAnnotationPayload>,
) -> AppResult<Response> {
    ensure_published_note(&state, &slug).await?;
    let username = require_public_user(&jar, &state)?;
    validate_offsets(payload.start_offset, payload.end_offset)?;
    let quote = payload.quote.trim().to_string();
    if quote.is_empty() {
        return Err(AppError::BadRequest("selected text cannot be empty".into()));
    }
    let color = normalize_color(payload.color)?;
    let comment = normalize_comment(payload.comment);
    if color.is_none() && comment.is_none() {
        return Err(AppError::BadRequest(
            "annotation must include a highlight color or comment".into(),
        ));
    }
    let visibility = normalize_visibility(payload.visibility, comment.is_some())?;
    let record = state.db.create_annotation(NewAnnotation {
        username: &username,
        note_slug: &slug,
        start_offset: payload.start_offset,
        end_offset: payload.end_offset,
        quote: &quote,
        color: color.as_deref(),
        comment: comment.as_deref(),
        visibility: &visibility,
    })?;
    Ok((StatusCode::CREATED, Json(record)).into_response())
}

pub async fn update_annotation(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<UpdateAnnotationPayload>,
) -> AppResult<Json<NoteAnnotation>> {
    let username = require_public_user(&jar, &state)?;
    let color = normalize_color(payload.color)?;
    let comment = normalize_comment(payload.comment);
    if color.is_none() && comment.is_none() {
        return Err(AppError::BadRequest(
            "annotation update must include a highlight color or comment".into(),
        ));
    }
    let visibility = normalize_visibility(payload.visibility, comment.is_some())?;
    let annotation = state
        .db
        .update_annotation(
            id,
            &username,
            color.as_deref(),
            comment.as_deref(),
            &visibility,
        )?
        .ok_or_else(|| AppError::NotFound(format!("annotation {}", id)))?;
    Ok(Json(annotation))
}

pub async fn delete_annotation(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<StatusCode> {
    let username = require_public_user(&jar, &state)?;
    if !state.db.delete_annotation(id, &username)? {
        return Err(AppError::NotFound(format!("annotation {}", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn require_public_user(jar: &CookieJar, state: &AppState) -> AppResult<String> {
    auth::current_public_user(jar, state)?.ok_or(AppError::Unauthorized)
}

async fn ensure_published_note(state: &AppState, slug: &str) -> AppResult<()> {
    let site = state.site.read().await;
    let exists = site.note(slug).filter(|note| note.is_published()).is_some();
    if exists {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("note {}", slug)))
    }
}

fn normalize_next(next: Option<&str>, fallback: &str) -> String {
    let target = next.unwrap_or(fallback).trim();
    if target.starts_with('/') && !target.starts_with("//") {
        target.to_string()
    } else {
        fallback.to_string()
    }
}

fn normalize_color(color: Option<String>) -> AppResult<Option<String>> {
    let Some(color) = color.map(|value| value.trim().to_string()) else {
        return Ok(None);
    };
    if color.is_empty() {
        return Ok(None);
    }
    let valid = color.len() == 7
        && color.starts_with('#')
        && color.chars().skip(1).all(|ch| ch.is_ascii_hexdigit());
    if !valid {
        return Err(AppError::BadRequest("invalid highlight color".into()));
    }
    Ok(Some(color.to_ascii_lowercase()))
}

fn normalize_comment(comment: Option<String>) -> Option<String> {
    comment.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn normalize_visibility(value: Option<String>, has_comment: bool) -> AppResult<String> {
    if !has_comment {
        return Ok("private".into());
    }

    let visibility = value
        .unwrap_or_else(|| "private".into())
        .trim()
        .to_ascii_lowercase();

    match visibility.as_str() {
        "private" | "public" => Ok(visibility),
        _ => Err(AppError::BadRequest("invalid comment visibility".into())),
    }
}

fn validate_offsets(start_offset: usize, end_offset: usize) -> AppResult<()> {
    if end_offset <= start_offset {
        return Err(AppError::BadRequest("annotation range is invalid".into()));
    }
    Ok(())
}

fn render_account(
    state: &AppState,
    viewer: Option<String>,
    next: String,
    login_username: String,
    register_username: String,
    login_error: Option<String>,
    register_error: Option<String>,
) -> AppResult<Response> {
    AccountTemplate {
        site_name: state.config.site_name.clone(),
        current_user: viewer,
        next,
        login_username,
        register_username,
        login_error,
        register_error,
    }
    .render()
    .map(Html)
    .map(IntoResponse::into_response)
    .map_err(AppError::internal)
}
