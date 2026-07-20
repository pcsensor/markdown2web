use std::sync::OnceLock;
use std::time::{Duration, Instant};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    app::AppState,
    content::Note,
    error::{AppError, AppResult},
    search::index::search_notes,
    web::auth,
};

const TANGSHAN_LAT: f64 = 39.6309;
const TANGSHAN_LON: f64 = 118.1802;
const WEATHER_CACHE_TTL: Duration = Duration::from_secs(15 * 60);
const WEATHER_FALLBACK: &str = "河北唐山 · —°C · 天气获取中";

struct WeatherCache {
    line: String,
    fetched_at: Instant,
}

fn weather_cache() -> &'static Mutex<Option<WeatherCache>> {
    static CACHE: OnceLock<Mutex<Option<WeatherCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn wmo_weather_zh(code: i32) -> &'static str {
    match code {
        0 => "晴",
        1 => "大体晴朗",
        2 => "多云",
        3 => "阴",
        45 | 48 => "雾",
        51 | 53 | 55 => "毛毛雨",
        56 | 57 => "冻毛毛雨",
        61 | 63 | 65 => "雨",
        66 | 67 => "冻雨",
        71 | 73 | 75 | 77 => "雪",
        80 | 81 | 82 => "阵雨",
        85 | 86 => "阵雪",
        95 => "雷阵雨",
        96 | 99 => "雷暴伴冰雹",
        _ => "多变",
    }
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: Option<f64>,
    weather_code: Option<i32>,
}

async fn fetch_tangshan_location_line() -> String {
    {
        let guard = weather_cache().lock().await;
        if let Some(cache) = guard.as_ref() {
            if cache.fetched_at.elapsed() < WEATHER_CACHE_TTL {
                return cache.line.clone();
            }
        }
    }

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={TANGSHAN_LAT}&longitude={TANGSHAN_LON}&current=temperature_2m,weather_code&timezone=Asia%2FShanghai"
    );

    let line = match reqwest::Client::new()
        .get(&url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
    {
        Ok(response) => match response.json::<OpenMeteoResponse>().await {
            Ok(payload) => {
                let current = payload.current.unwrap_or(OpenMeteoCurrent {
                    temperature_2m: None,
                    weather_code: None,
                });
                let temp = current
                    .temperature_2m
                    .map(|t| format!("{:.0}°C", t))
                    .unwrap_or_else(|| "—°C".into());
                let desc = current
                    .weather_code
                    .map(wmo_weather_zh)
                    .unwrap_or("多变");
                format!("河北唐山 · {temp} · {desc}")
            }
            Err(_) => WEATHER_FALLBACK.into(),
        },
        Err(_) => WEATHER_FALLBACK.into(),
    };

    let mut guard = weather_cache().lock().await;
    *guard = Some(WeatherCache {
        line: line.clone(),
        fetched_at: Instant::now(),
    });
    line
}

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
    location_line: String,
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
    let location_line = fetch_tangshan_location_line().await;
    render(HomeTemplate {
        site_name: state.config.site_name.clone(),
        notes: site.published_notes().into_iter().take(8).collect(),
        build_message: site.build_message,
        location_line,
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
    let (viewer, is_admin) = auth::current_viewer(&jar, &state)?.unzip();
    let is_admin = is_admin.unwrap_or(false);
    render(NoteTemplate {
        site_name: state.config.site_name.clone(),
        note,
        backlinks,
        annotations_enabled: viewer.is_some(),
        viewer_username: viewer.clone().unwrap_or_default(),
        viewer,
        is_admin,
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
