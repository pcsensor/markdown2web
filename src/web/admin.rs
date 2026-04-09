use std::fs;

use askama::Template;
use axum::{
    Form,
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
    store::filesystem,
    web::auth,
};

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    site_name: String,
    username: String,
    error: Option<String>,
    success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    site_name: String,
    username: String,
    notes: Vec<Note>,
    build_events: Vec<crate::store::sqlite::BuildEvent>,
    password_error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/note_edit.html")]
struct NoteEditTemplate {
    site_name: String,
    username: String,
    mode: String,
    note: EditableNote,
}

#[derive(Debug, Clone, Default)]
struct EditableNote {
    title: String,
    slug: String,
    summary: String,
    tags: String,
    status: String,
    aliases: String,
    body: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct SaveNoteForm {
    title: String,
    slug: Option<String>,
    summary: Option<String>,
    tags: Option<String>,
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
    })
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    if !state.db.verify_user(&form.username, &form.password)? {
        return render(LoginTemplate {
            site_name: state.config.site_name.clone(),
            username: form.username,
            error: Some("Invalid username or password".into()),
            success: None,
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
        title: Some(form.title),
        slug: Some(slug.clone()),
        summary: form.summary,
        tags: csv_to_vec(form.tags.as_deref().unwrap_or_default()),
        status: Some(form.status.unwrap_or_else(|| "published".into())),
        aliases: csv_to_vec(form.aliases.as_deref().unwrap_or_default()),
    };
    let contents = compose_markdown(&front_matter, &form.body)?;
    filesystem::write_note(&state.config, &slug, &contents)?;
    state
        .build_service
        .rebuild(format!("admin save {}", slug))
        .await?;
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
    while let Some(field) = multipart.next_field().await.map_err(AppError::internal)? {
        let filename = field.file_name().unwrap_or("upload.md").to_string();
        if !filename.ends_with(".md") && !filename.ends_with(".markdown") {
            return Err(AppError::BadRequest(
                "only markdown uploads are allowed".into(),
            ));
        }
        let bytes = field.bytes().await.map_err(AppError::internal)?;
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
    state.build_service.rebuild("markdown upload").await?;
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
    while let Some(field) = multipart.next_field().await.map_err(AppError::internal)? {
        let filename = field.file_name().unwrap_or("upload.bin").to_string();
        if !allowed_asset_filename(&filename) {
            return Err(AppError::BadRequest(
                "unsupported asset type; allowlisted examples: png, jpg, webp, svg, pdf, zip, txt"
                    .into(),
            ));
        }
        let bytes = field.bytes().await.map_err(AppError::internal)?;
        if bytes.len() > state.config.upload_limit_mb * 1024 * 1024 {
            return Err(AppError::BadRequest("upload too large".into()));
        }
        filesystem::write_asset(&state.config, &filename, &bytes)?;
    }
    state.build_service.rebuild("asset upload").await?;
    Ok(Redirect::to("/admin").into_response())
}

pub async fn rebuild_site(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let Some(_user) = auth::current_user(&jar, &state)? else {
        return Ok(Redirect::to("/admin/login").into_response());
    };
    state.build_service.rebuild("manual rebuild").await?;
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
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf" | "txt" | "zip"
    )
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
    })
}

fn render<T: Template>(template: T) -> AppResult<Response> {
    template
        .render()
        .map(Html)
        .map(IntoResponse::into_response)
        .map_err(AppError::internal)
}
