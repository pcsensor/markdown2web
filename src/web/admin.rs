use std::fs;

use askama::Template;
use axum::{
    Form, Json,
    extract::{Multipart, Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    app::AppState,
    content::{
        Note,
        front_matter::{FrontMatter, compose_markdown, parse_front_matter},
        markdown::slugify,
    },
    error::{AppError, AppResult},
    store::{
        filesystem,
        sqlite::{BuildEvent, ManagedPublicUser},
    },
    web::{auth, turnstile},
};

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    site_name: String,
    username: String,
    error: Option<String>,
    success: Option<String>,
    turnstile_site_key: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    site_name: String,
    username: String,
    notes: Vec<Note>,
    build_events: Vec<BuildEvent>,
    password_error: Option<String>,
    public_user_count: usize,
}

#[derive(Template)]
#[template(path = "admin/note_edit.html")]
struct NoteEditTemplate {
    site_name: String,
    username: String,
    mode: String,
    note: EditableNote,
}

#[derive(Template)]
#[template(path = "admin/users.html")]
struct UsersTemplate {
    site_name: String,
    users: Vec<ManagedPublicUser>,
    user_count: usize,
    create_username: String,
    create_error: Option<String>,
    success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/user_edit.html")]
struct UserEditTemplate {
    site_name: String,
    managed_user: ManagedPublicUser,
    form_username: String,
    update_error: Option<String>,
    success: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct EditableNote {
    title: String,
    slug: String,
    summary: String,
    tags: String,
    category: String,
    status: String,
    aliases: String,
    body: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    #[serde(default)]
    #[serde(rename = "cf-turnstile-response")]
    cf_turnstile_response: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveNoteForm {
    title: String,
    slug: Option<String>,
    summary: Option<String>,
    tags: Option<String>,
    category: Option<String>,
    status: Option<String>,
    aliases: Option<String>,
    body: String,
}

#[derive(Default, Deserialize)]
pub struct LoginStatusQuery {
    password: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

#[derive(Default, Deserialize)]
pub struct UserManagementStatusQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePublicUserForm {
    username: String,
    password: String,
    confirm_password: String,
}

#[derive(Deserialize)]
pub struct UpdatePublicUserForm {
    username: String,
    new_password: String,
    confirm_password: String,
}

pub async fn login_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<LoginStatusQuery>,
) -> AppResult<Response> {
    if auth::current_user(&jar, &state)?.is_some() {
        return Ok(Redirect::to("/admin").into_response());
    }
    render(LoginTemplate {
        site_name: state.config.site_name.clone(),
        username: state.config.admin_username.clone(),
        error: None,
        success: match query.password.as_deref() {
            Some("updated") => Some("密码已更新，请使用新密码重新登录。".into()),
            _ => None,
        },
        turnstile_site_key: state.config.turnstile_site_key.clone(),
    })
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    let token = form.cf_turnstile_response.as_deref().unwrap_or_default();
    match turnstile::verify_turnstile(token, &state.config.turnstile_secret_key, None).await {
        Ok(true) => {}
        Ok(false) => {
            return render(LoginTemplate {
                site_name: state.config.site_name.clone(),
                username: form.username,
                error: Some("人机验证失败，请重试。".into()),
                success: None,
                turnstile_site_key: state.config.turnstile_site_key.clone(),
            });
        }
        Err(err) => {
            return render(LoginTemplate {
                site_name: state.config.site_name.clone(),
                username: form.username,
                error: Some(format!("人机验证异常：{err}")),
                success: None,
                turnstile_site_key: state.config.turnstile_site_key.clone(),
            });
        }
    }

    if !state.db.verify_user(&form.username, &form.password)? {
        return render(LoginTemplate {
            site_name: state.config.site_name.clone(),
            username: form.username,
            error: Some("Invalid username or password".into()),
            success: None,
            turnstile_site_key: state.config.turnstile_site_key.clone(),
        });
    }
    let token = state.db.create_session(&form.username)?;
    Ok((
        jar.add(auth::build_session_cookie(token)),
        Redirect::to("/admin"),
    )
        .into_response())
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    if let Some(token) = auth::session_token(&jar) {
        let _ = state.db.delete_session(&token);
    }
    Ok((
        jar.remove(auth::clear_session_cookie()),
        Redirect::to("/admin/login"),
    )
        .into_response())
}

pub async fn dashboard(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    dashboard_response(&state, user, None).await
}

pub async fn change_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<ChangePasswordForm>,
) -> AppResult<Response> {
    const MIN_PASSWORD_LENGTH: usize = 8;

    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    if !state.db.verify_user(&user, &form.current_password)? {
        return dashboard_response(&state, user, Some("当前密码不正确。".into())).await;
    }
    if form.new_password.trim().is_empty() {
        return dashboard_response(&state, user, Some("新密码不能为空。".into())).await;
    }
    if form.new_password.len() < MIN_PASSWORD_LENGTH {
        return dashboard_response(
            &state,
            user,
            Some(format!("新密码至少需要 {MIN_PASSWORD_LENGTH} 个字符。")),
        )
        .await;
    }
    if form.new_password != form.confirm_password {
        return dashboard_response(&state, user, Some("两次输入的新密码不一致。".into())).await;
    }
    if form.new_password == form.current_password {
        return dashboard_response(&state, user, Some("新密码不能与当前密码相同。".into())).await;
    }
    if !state.db.update_password(&user, &form.new_password)? {
        return Err(AppError::Unauthorized);
    }
    state.db.delete_sessions_for_user(&user)?;
    Ok((
        jar.remove(auth::clear_session_cookie()),
        Redirect::to("/admin/login?password=updated"),
    )
        .into_response())
}

pub async fn users_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<UserManagementStatusQuery>,
) -> AppResult<Response> {
    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    users_response(
        &state,
        user,
        String::new(),
        None,
        users_success_message(query.status.as_deref()),
    )
    .await
}

pub async fn create_public_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CreatePublicUserForm>,
) -> AppResult<Response> {
    const MIN_PASSWORD_LENGTH: usize = 8;

    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    let username = form.username.trim().to_string();
    if username.is_empty() {
        return users_response(
            &state,
            user,
            username,
            Some("用户名不能为空。".into()),
            None,
        )
        .await;
    }
    if form.password.len() < MIN_PASSWORD_LENGTH {
        return users_response(
            &state,
            user,
            username,
            Some(format!("密码至少需要 {MIN_PASSWORD_LENGTH} 个字符。")),
            None,
        )
        .await;
    }
    if form.password != form.confirm_password {
        return users_response(
            &state,
            user,
            username,
            Some("两次输入的密码不一致。".into()),
            None,
        )
        .await;
    }
    if !state.db.create_public_user(&username, &form.password)? {
        return users_response(
            &state,
            user,
            username,
            Some("该用户名已被注册。".into()),
            None,
        )
        .await;
    }

    let route_key = state
        .db
        .public_user_summary(&username)?
        .map(|managed_user| managed_user.route_key)
        .ok_or_else(|| AppError::NotFound(format!("user {}", username)))?;
    Ok(Redirect::to(&format!("/admin/users/{route_key}?status=created")).into_response())
}

pub async fn user_detail_page(
    Path(username): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<UserManagementStatusQuery>,
) -> AppResult<Response> {
    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    user_detail_response(
        &state,
        user,
        &username,
        username.clone(),
        None,
        user_detail_success_message(query.status.as_deref()),
    )
    .await
}

pub async fn update_public_user(
    Path(current_username): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<UpdatePublicUserForm>,
) -> AppResult<Response> {
    const MIN_PASSWORD_LENGTH: usize = 8;

    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    let next_username = form.username.trim().to_string();
    if next_username.is_empty() {
        return user_detail_response(
            &state,
            user,
            &current_username,
            next_username,
            Some("用户名不能为空。".into()),
            None,
        )
        .await;
    }

    let next_password = if form.new_password.is_empty() {
        None
    } else {
        if form.new_password.len() < MIN_PASSWORD_LENGTH {
            return user_detail_response(
                &state,
                user,
                &current_username,
                next_username,
                Some(format!("新密码至少需要 {MIN_PASSWORD_LENGTH} 个字符。")),
                None,
            )
            .await;
        }
        if form.new_password != form.confirm_password {
            return user_detail_response(
                &state,
                user,
                &current_username,
                next_username,
                Some("两次输入的新密码不一致。".into()),
                None,
            )
            .await;
        }
        Some(form.new_password.as_str())
    };

    if current_username == next_username && next_password.is_none() {
        return user_detail_response(
            &state,
            user,
            &current_username,
            next_username,
            Some("请至少修改用户名或重置密码。".into()),
            None,
        )
        .await;
    }

    let updated_username =
        match state
            .db
            .update_public_user(&current_username, &next_username, next_password)
        {
            Ok(Some(updated_username)) => updated_username,
            Ok(None) => return Err(AppError::NotFound(format!("user {}", current_username))),
            Err(AppError::BadRequest(message)) => {
                return user_detail_response(
                    &state,
                    user,
                    &current_username,
                    next_username,
                    Some(message),
                    None,
                )
                .await;
            }
            Err(error) => return Err(error),
        };

    let route_key = state
        .db
        .public_user_summary(&updated_username)?
        .map(|managed_user| managed_user.route_key)
        .ok_or_else(|| AppError::NotFound(format!("user {}", updated_username)))?;
    Ok(Redirect::to(&format!("/admin/users/{route_key}?status=updated")).into_response())
}

pub async fn delete_public_user(
    Path(username): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    if !state.db.delete_public_user(&username)? {
        return Err(AppError::NotFound(format!("user {}", username)));
    }
    Ok(Redirect::to("/admin/users?status=deleted").into_response())
}

pub async fn new_note_page(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    render(NoteEditTemplate {
        site_name: state.config.site_name.clone(),
        username: user,
        mode: "Create".into(),
        note: EditableNote {
            status: "published".into(),
            ..Default::default()
        },
    })
}

pub async fn edit_note_page(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Response> {
    let Some(user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    let site = state.site.read().await.clone();
    let note = site
        .note(&slug)
        .ok_or_else(|| AppError::NotFound(format!("note {}", slug)))?;
    let raw = fs::read_to_string(&note.source_path)?;
    let (front_matter, body) = parse_front_matter(&raw)?;
    render(NoteEditTemplate {
        site_name: state.config.site_name.clone(),
        username: user,
        mode: "Edit".into(),
        note: EditableNote {
            title: front_matter.title.unwrap_or(note.title),
            slug: front_matter.slug.unwrap_or(note.slug),
            summary: front_matter.summary.unwrap_or(note.summary),
            tags: front_matter.tags.join(", "),
            category: front_matter.category.join(", "),
            status: front_matter.status.unwrap_or(note.status),
            aliases: front_matter.aliases.join(", "),
            body,
        },
    })
}

pub async fn save_note(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<SaveNoteForm>,
) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    let slug = form
        .slug
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| slugify(&form.title));
    let front_matter = FrontMatter {
        title: Some(form.title.clone()),
        slug: Some(slug.clone()),
        summary: form.summary,
        tags: csv_to_vec(form.tags.as_deref().unwrap_or_default()),
        category: csv_to_vec(form.category.as_deref().unwrap_or_default()),
        status: Some(form.status.unwrap_or_else(|| "published".into())),
        aliases: csv_to_vec(form.aliases.as_deref().unwrap_or_default()),
    };
    let contents = compose_markdown(&front_matter, &form.body)?;
    filesystem::write_note(&state.config, &slug, &contents)?;
    let summary = state
        .build_service
        .rebuild(format!("admin save {}", slug))
        .await?;
    if !summary.media_jobs.is_empty() {
        state.build_service.clone().spawn_media_worker(summary.media_jobs).await;
    }
    Ok(Redirect::to("/admin").into_response())
}

pub async fn upload_markdown(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(multipart_upload_error)?
    {
        let filename = field.file_name().unwrap_or("upload.md").to_string();
        if !filename.ends_with(".md") && !filename.ends_with(".markdown") {
            return Err(AppError::BadRequest(
                "only markdown uploads are allowed".into(),
            ));
        }
        let bytes = field.bytes().await.map_err(multipart_upload_error)?;
        if bytes.len() > state.config.upload_limit_mb * 1024 * 1024 {
            return Err(AppError::BadRequest("upload too large".into()));
        }
        let slug = slugify(
            filename
                .trim_end_matches(".markdown")
                .trim_end_matches(".md"),
        );
        filesystem::write_note(
            &state.config,
            &slug,
            std::str::from_utf8(&bytes).map_err(AppError::internal)?,
        )?;
    }
    let summary = state.build_service.rebuild("markdown upload").await?;
    if !summary.media_jobs.is_empty() {
        state.build_service.clone().spawn_media_worker(summary.media_jobs).await;
    }
    Ok(Redirect::to("/admin").into_response())
}

pub async fn upload_asset(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(multipart_upload_error)?
    {
        let filename = field.file_name().unwrap_or("upload.bin").to_string();
        if !allowed_asset_filename(&filename) {
            return Err(AppError::BadRequest(
                "unsupported asset type; allowlisted examples: png, jpg, webp, svg, mp3, mp4, pdf, zip, txt"
                    .into(),
            ));
        }
        let bytes = field.bytes().await.map_err(multipart_upload_error)?;
        if bytes.len() > state.config.upload_limit_mb * 1024 * 1024 {
            return Err(AppError::BadRequest("upload too large".into()));
        }
        filesystem::write_asset(&state.config, &filename, &bytes)?;
    }
    let summary = state.build_service.rebuild("asset upload").await?;
    if !summary.media_jobs.is_empty() {
        state.build_service.clone().spawn_media_worker(summary.media_jobs).await;
    }
    Ok(Redirect::to("/admin").into_response())
}

pub async fn rebuild_site(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    let summary = state.build_service.rebuild("manual rebuild").await?;
    if !summary.media_jobs.is_empty() {
        state.build_service.clone().spawn_media_worker(summary.media_jobs).await;
    }
    Ok(Redirect::to("/admin").into_response())
}

fn csv_to_vec(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn allowed_asset_filename(filename: &str) -> bool {
    let ext = filename
        .rsplit('.')
        .next()
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "mp3" | "mp4" | "pdf" | "txt" | "zip"
    )
}

pub async fn get_build_progress(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Json<crate::build::pipeline::BuildProgress>> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Err(AppError::Unauthorized);
    };
    let progress = state.build_service.progress.read().await;
    Ok(Json(progress.clone()))
}

fn multipart_upload_error(error: impl std::fmt::Display) -> AppError {
    AppError::BadRequest(format!(
        "invalid multipart upload or upload too large; check M2W_UPLOAD_LIMIT_MB: {error}"
    ))
}

async fn dashboard_response(
    state: &AppState,
    user: String,
    password_error: Option<String>,
) -> AppResult<Response> {
    let site = state.site.read().await.clone();
    render(DashboardTemplate {
        site_name: state.config.site_name.clone(),
        username: user,
        notes: site.all_notes(),
        build_events: state.db.recent_builds(12)?,
        password_error,
        public_user_count: state.db.public_user_count()?,
    })
}

async fn users_response(
    state: &AppState,
    _admin_user: String,
    create_username: String,
    create_error: Option<String>,
    success: Option<String>,
) -> AppResult<Response> {
    let users = state.db.list_public_users()?;
    let user_count = users.len();
    render(UsersTemplate {
        site_name: state.config.site_name.clone(),
        users,
        user_count,
        create_username,
        create_error,
        success,
    })
}

async fn user_detail_response(
    state: &AppState,
    _admin_user: String,
    managed_username: &str,
    form_username: String,
    update_error: Option<String>,
    success: Option<String>,
) -> AppResult<Response> {
    let managed_user = state
        .db
        .public_user_summary(managed_username)?
        .ok_or_else(|| AppError::NotFound(format!("user {}", managed_username)))?;
    render(UserEditTemplate {
        site_name: state.config.site_name.clone(),
        managed_user,
        form_username,
        update_error,
        success,
    })
}

fn users_success_message(status: Option<&str>) -> Option<String> {
    match status {
        Some("deleted") => Some("用户已删除，相关公开登录会话与评论数据也已清理。".into()),
        _ => None,
    }
}

fn user_detail_success_message(status: Option<&str>) -> Option<String> {
    match status {
        Some("created") => Some("用户已创建。你现在可以继续调整用户名或重置密码。".into()),
        Some("updated") => Some("账号变更已保存，该用户需要重新登录。".into()),
        _ => None,
    }
}

fn render<T: Template>(template: T) -> AppResult<Response> {
    template
        .render()
        .map(Html)
        .map(IntoResponse::into_response)
        .map_err(AppError::internal)
}
