use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    app::AppState,
    content::Note,
    error::{AppError, AppResult},
    search::index::search_notes,
    web::auth,
};

#[derive(Debug, Clone)]
struct LinkView {
    slug: String,
    title: String,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    site_name: String,
    notes: Vec<Note>,
    build_message: String,
}

#[derive(Template)]
#[template(path = "notes.html")]
struct NotesTemplate {
    site_name: String,
    title: String,
    notes: Vec<Note>,
    categories: Vec<String>,
}

#[derive(Template)]
#[template(path = "note.html")]
struct NoteTemplate {
    site_name: String,
    note: Note,
    backlinks: Vec<LinkView>,
    viewer: Option<String>,
    viewer_username: String,
    annotations_enabled: bool,
    is_admin: bool,
    csrf_token: String,
}

#[derive(Template)]
#[template(path = "tag.html")]
struct TagTemplate {
    site_name: String,
    tag: String,
    notes: Vec<Note>,
}

// Category template: mirrors TagTemplate but for category-based notes view
#[derive(Template)]
#[template(path = "category.html")]
struct CategoryTemplate {
    site_name: String,
    category: String,
    notes: Vec<Note>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    site_name: String,
    query: String,
    notes: Vec<Note>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    render(HomeTemplate {
        site_name: state.config.site_name.clone(),
        notes: site.published_notes().into_iter().take(8).collect(),
        build_message: site.build_message,
    })
}

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let site = state.site.read().await;
    format!("ok:{}", site.notes.len())
}

pub async fn notes_index(State(state): State<AppState>) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    let notes = site.published_notes();
    let mut categories: Vec<String> = notes
        .iter()
        .flat_map(|note| note.category.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    categories.sort();
    render(NotesTemplate {
        site_name: state.config.site_name.clone(),
        title: "All Notes".into(),
        notes,
        categories,
    })
}

pub async fn note_detail(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    let note = site
        .note(&slug)
        .filter(|note| note.is_published())
        .ok_or_else(|| AppError::NotFound(format!("note {}", slug)))?;
    let backlinks = site
        .backlinks
        .get(&slug)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item_slug| {
            site.notes.get(&item_slug).map(|note| LinkView {
                slug: item_slug,
                title: note.title.clone(),
            })
        })
        .collect();
    let viewer_session = auth::current_viewer_session(&jar, &state)?;
    let viewer = viewer_session
        .as_ref()
        .map(|session| session.username.clone());
    let is_admin = viewer_session
        .as_ref()
        .map(|session| session.is_admin)
        .unwrap_or(false);
    let csrf_token = viewer_session
        .as_ref()
        .map(|session| session.csrf_token.clone())
        .unwrap_or_default();
    render(NoteTemplate {
        site_name: state.config.site_name.clone(),
        note,
        backlinks,
        annotations_enabled: viewer.is_some(),
        viewer_username: viewer.clone().unwrap_or_default(),
        viewer,
        is_admin,
        csrf_token,
    })
}

pub async fn tag_detail(
    Path(tag): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    let notes = site
        .tags
        .get(&tag)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|slug| site.note(&slug))
        .filter(|note| note.is_published())
        .collect();
    render(TagTemplate {
        site_name: state.config.site_name.clone(),
        tag,
        notes,
    })
}

// Category detail page: show all published notes for a given category
pub async fn category_detail(
    Path(category): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    let notes = site
        .published_notes()
        .into_iter()
        .filter(|note| note.category.contains(&category))
        .collect();
    render(CategoryTemplate {
        site_name: state.config.site_name.clone(),
        category,
        notes,
    })
}

pub async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> AppResult<Html<String>> {
    let site = state.site.read().await.clone();
    let query_value = query.q.unwrap_or_default();
    render(SearchTemplate {
        site_name: state.config.site_name.clone(),
        notes: search_notes(&site, &query_value),
        query: query_value,
    })
}

fn render<T: Template>(template: T) -> AppResult<Html<String>> {
    template.render().map(Html).map_err(AppError::internal)
}
