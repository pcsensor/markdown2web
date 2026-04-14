use askama::Template;
use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    store::sqlite::{NewAnnotation, NewVideoDanmaku, NoteAnnotation, VideoDanmaku},
    web::{auth, csrf, rate_limit, turnstile},
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
    turnstile_enabled: bool,
    turnstile_site_key: String,
    csrf_token: String,
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
    #[serde(default)]
    #[serde(rename = "_csrf")]
    csrf_token: String,
    #[serde(default)]
    #[serde(rename = "cf-turnstile-response")]
    cf_turnstile_response: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    next: Option<String>,
    #[serde(default)]
    #[serde(rename = "_csrf")]
    csrf_token: String,
    #[serde(default)]
    #[serde(rename = "cf-turnstile-response")]
    cf_turnstile_response: Option<String>,
}

#[derive(Deserialize)]
pub struct LogoutForm {
    next: Option<String>,
    #[serde(default)]
    #[serde(rename = "_csrf")]
    csrf_token: String,
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

#[derive(Debug, Deserialize)]
pub struct DanmakuQuery {
    video_src: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDanmakuPayload {
    video_src: String,
    time_ms: i64,
    body: String,
    color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DanmakuListResponse {
    danmaku: Vec<VideoDanmaku>,
}

pub async fn account_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AccountQuery>,
) -> AppResult<Response> {
    let next = normalize_next(query.next.as_deref(), "/account");
    if let Some(session) = auth::current_public_session(&jar, &state)? {
        return render_account(
            &state,
            Some(session.username),
            next,
            String::new(),
            String::new(),
            None,
            None,
            session.csrf_token,
        );
    }

    let csrf_token = csrf::generate_token();
    let response = render_account(
        &state,
        None,
        next,
        String::new(),
        String::new(),
        None,
        None,
        csrf_token.clone(),
    )?;
    Ok((
        jar.add(csrf::build_pre_auth_cookie(csrf_token, &state.config)),
        response,
    )
        .into_response())
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<RegisterForm>,
) -> AppResult<Response> {
    let next = normalize_next(form.next.as_deref(), "/account");
    if auth::current_public_user(&jar, &state)?.is_some() {
        return Ok(Redirect::to(&next).into_response());
    }
    csrf::verify_pre_auth(&jar, &form.csrf_token)?;
    rate_limit::check(
        &state,
        "account-register",
        &form.username,
        &headers,
        rate_limit::PUBLIC_AUTH_LIMIT,
        rate_limit::PUBLIC_AUTH_WINDOW_SECS,
    )?;

    let token = form.cf_turnstile_response.as_deref().unwrap_or_default();
    match turnstile::verify_turnstile(token, &state.config, None).await {
        Ok(true) => {}
        Ok(false) => {
            return render_account(
                &state,
                None,
                next,
                String::new(),
                form.username.trim().to_string(),
                None,
                Some("人机验证失败，请重试。".into()),
                form.csrf_token,
            );
        }
        Err(err) => {
            return render_account(
                &state,
                None,
                next,
                String::new(),
                form.username.trim().to_string(),
                None,
                Some(format!("人机验证异常：{err}")),
                form.csrf_token,
            );
        }
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
            form.csrf_token,
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
            form.csrf_token,
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
            form.csrf_token,
        );
    }
    let session = state
        .db
        .create_public_session(&username, state.config.session_ttl_hours)?;
    Ok((
        jar.remove(csrf::clear_pre_auth_cookie())
            .add(auth::build_user_session_cookie(
                session.token,
                &state.config,
            )),
        Redirect::to(&next),
    )
        .into_response())
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    let next = normalize_next(form.next.as_deref(), "/account");
    if auth::current_public_user(&jar, &state)?.is_some() {
        return Ok(Redirect::to(&next).into_response());
    }
    csrf::verify_pre_auth(&jar, &form.csrf_token)?;
    rate_limit::check(
        &state,
        "account-login",
        &form.username,
        &headers,
        rate_limit::PUBLIC_AUTH_LIMIT,
        rate_limit::PUBLIC_AUTH_WINDOW_SECS,
    )?;

    let token = form.cf_turnstile_response.as_deref().unwrap_or_default();
    match turnstile::verify_turnstile(token, &state.config, None).await {
        Ok(true) => {}
        Ok(false) => {
            return render_account(
                &state,
                None,
                next,
                form.username.trim().to_string(),
                String::new(),
                Some("人机验证失败，请重试。".into()),
                None,
                form.csrf_token,
            );
        }
        Err(err) => {
            return render_account(
                &state,
                None,
                next,
                form.username.trim().to_string(),
                String::new(),
                Some(format!("人机验证异常：{err}")),
                None,
                form.csrf_token,
            );
        }
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
            form.csrf_token,
        );
    }

    let session = state
        .db
        .create_public_session(&username, state.config.session_ttl_hours)?;
    Ok((
        jar.remove(csrf::clear_pre_auth_cookie())
            .add(auth::build_user_session_cookie(
                session.token,
                &state.config,
            )),
        Redirect::to(&next),
    )
        .into_response())
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LogoutForm>,
) -> AppResult<Response> {
    let Some(session) = auth::current_public_session(&jar, &state)? else {
        return Ok(Redirect::to("/account").into_response());
    };
    csrf::verify_form(&session.csrf_token, &form.csrf_token)?;
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
    let (username, _is_admin) = auth::current_viewer(&jar, &state)?.unwrap_or_default();
    let viewer = if username.is_empty() {
        None
    } else {
        Some(username)
    };
    Ok(Json(AnnotationListResponse {
        annotations: state
            .db
            .list_visible_annotations(&slug, viewer.as_deref())?,
    }))
}

pub async fn create_annotation(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<CreateAnnotationPayload>,
) -> AppResult<Response> {
    ensure_published_note(&state, &slug).await?;
    let session = auth::current_viewer_session(&jar, &state)?.ok_or(AppError::Unauthorized)?;
    csrf::verify_header(&headers, &session.csrf_token)?;
    rate_limit::check(
        &state,
        "api-annotation",
        &session.username,
        &headers,
        rate_limit::API_WRITE_LIMIT,
        rate_limit::API_WRITE_WINDOW_SECS,
    )?;
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
        username: &session.username,
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
    headers: HeaderMap,
    Json(payload): Json<UpdateAnnotationPayload>,
) -> AppResult<Json<NoteAnnotation>> {
    let session = auth::current_viewer_session(&jar, &state)?.ok_or(AppError::Unauthorized)?;
    csrf::verify_header(&headers, &session.csrf_token)?;
    rate_limit::check(
        &state,
        "api-annotation",
        &session.username,
        &headers,
        rate_limit::API_WRITE_LIMIT,
        rate_limit::API_WRITE_WINDOW_SECS,
    )?;
    let color = normalize_color(payload.color)?;
    let comment = normalize_comment(payload.comment);
    if color.is_none() && comment.is_none() {
        return Err(AppError::BadRequest(
            "annotation update must include a highlight color or comment".into(),
        ));
    }
    let visibility = normalize_visibility(payload.visibility, comment.is_some())?;
    let annotation = if session.is_admin {
        state.db.update_annotation_by_admin(
            id,
            color.as_deref(),
            comment.as_deref(),
            &visibility,
        )?
    } else {
        state.db.update_annotation(
            id,
            &session.username,
            color.as_deref(),
            comment.as_deref(),
            &visibility,
        )?
    }
    .ok_or_else(|| AppError::NotFound(format!("annotation {}", id)))?;
    Ok(Json(annotation))
}

pub async fn delete_annotation(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let session = auth::current_viewer_session(&jar, &state)?.ok_or(AppError::Unauthorized)?;
    csrf::verify_header(&headers, &session.csrf_token)?;
    rate_limit::check(
        &state,
        "api-annotation",
        &session.username,
        &headers,
        rate_limit::API_WRITE_LIMIT,
        rate_limit::API_WRITE_WINDOW_SECS,
    )?;
    let deleted = if session.is_admin {
        state.db.delete_annotation_by_admin(id)?
    } else {
        state.db.delete_annotation(id, &session.username)?
    };
    if !deleted {
        return Err(AppError::NotFound(format!("annotation {}", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_danmaku(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    _jar: CookieJar,
    Query(query): Query<DanmakuQuery>,
) -> AppResult<Json<DanmakuListResponse>> {
    ensure_published_note(&state, &slug).await?;
    let (full_src, filename) = normalize_video_src(query.video_src)?;
    Ok(Json(DanmakuListResponse {
        danmaku: state.db.list_video_danmaku(&slug, &full_src, &filename)?,
    }))
}

pub async fn create_danmaku(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateDanmakuPayload>,
) -> AppResult<Response> {
    ensure_published_note(&state, &slug).await?;
    let (username, _is_admin) =
        auth::current_viewer(&jar, &state)?.ok_or(AppError::Unauthorized)?;
    let (full_src, _filename) = normalize_video_src(payload.video_src)?;
    let body = normalize_danmaku_body(payload.body)?;
    let color = normalize_color(payload.color)?.unwrap_or_else(|| "#ffffff".into());
    validate_danmaku_time(payload.time_ms)?;
    let record = state.db.create_video_danmaku(NewVideoDanmaku {
        username: &username,
        note_slug: &slug,
        video_src: &full_src,
        time_ms: payload.time_ms,
        body: &body,
        color: &color,
    })?;
    Ok((StatusCode::CREATED, Json(record)).into_response())
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

fn normalize_video_src(value: String) -> AppResult<(String, String)> {
    let value = value.trim();
    if value.is_empty() || value.len() > 512 {
        return Err(AppError::BadRequest("video source is invalid".into()));
    }

    // Keep the generated per-video suffix (`#0`, `#1`, ...) so repeated
    // embeds of the same asset do not share one danmaku timeline.
    let (path, fragment) = value.split_once('#').unwrap_or((value, ""));

    if !path.starts_with("/assets/") {
        return Err(AppError::BadRequest(
            "video source must be a site asset".into(),
        ));
    }

    let fragment_suffix = if fragment.is_empty() {
        String::new()
    } else {
        format!("#{fragment}")
    };
    let full_src = format!("{path}{fragment_suffix}");

    // 提取原始文件名（剥离路径和可能的哈希前缀），同时保留实例后缀。
    let asset_name = path.strip_prefix("/assets/").unwrap_or(path);
    let filename = if asset_name.len() > 13 && asset_name.as_bytes()[12] == b'-' {
        &asset_name[13..]
    } else {
        asset_name
    };
    let filename = format!("{filename}{fragment_suffix}");

    Ok((full_src, filename))
}

fn normalize_danmaku_body(value: String) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::BadRequest("danmaku cannot be empty".into()));
    }
    if value.chars().count() > 80 {
        return Err(AppError::BadRequest("danmaku is too long".into()));
    }
    Ok(value.into())
}

fn validate_danmaku_time(time_ms: i64) -> AppResult<()> {
    if !(0..=24 * 60 * 60 * 1000).contains(&time_ms) {
        return Err(AppError::BadRequest("danmaku time is invalid".into()));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_account(
    state: &AppState,
    viewer: Option<String>,
    next: String,
    login_username: String,
    register_username: String,
    login_error: Option<String>,
    register_error: Option<String>,
    csrf_token: String,
) -> AppResult<Response> {
    AccountTemplate {
        site_name: state.config.site_name.clone(),
        current_user: viewer,
        next,
        login_username,
        register_username,
        login_error,
        register_error,
        turnstile_enabled: state.config.turnstile_enabled,
        turnstile_site_key: state.config.turnstile_site_key.clone(),
        csrf_token,
    }
    .render()
    .map(Html)
    .map(IntoResponse::into_response)
    .map_err(AppError::internal)
}
