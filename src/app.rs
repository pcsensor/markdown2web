use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post, put},
};
use tokio::sync::{Mutex, RwLock};
use tower_http::services::ServeDir;

use crate::{
    build::pipeline::BuildService,
    config::AppConfig,
    content::SiteData,
    error::AppResult,
    store::{filesystem, sqlite::AppDatabase},
    web::{account, admin, public},
};

pub type WatcherHandle = Arc<std::sync::Mutex<Option<notify::RecommendedWatcher>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Arc<AppDatabase>,
    pub site: Arc<RwLock<SiteData>>,
    pub build_service: Arc<BuildService>,
    watcher: Arc<Mutex<Option<WatcherHandle>>>,
}

impl AppState {
    pub async fn bootstrap(config: AppConfig, db: Arc<AppDatabase>) -> AppResult<Self> {
        filesystem::ensure_sample_content(&config)?;
        let site = Arc::new(RwLock::new(SiteData::default()));
        let build_service = Arc::new(BuildService::new(config.clone(), db.clone(), site.clone())?);
        build_service.rebuild("startup").await?;
        Ok(Self {
            config,
            db,
            site,
            build_service,
            watcher: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn set_watcher_handle(&self, watcher: WatcherHandle) {
        let mut guard = self.watcher.lock().await;
        *guard = Some(watcher);
    }
}

pub fn build_router(state: AppState) -> Router {
    let static_service = ServeDir::new("static");
    let assets_service = ServeDir::new(state.config.generated_assets_dir.clone());
    let upload_limit_bytes = state
        .config
        .upload_limit_mb
        .saturating_add(1)
        .saturating_mul(1024 * 1024);

    Router::new()
        .route("/", get(public::home))
        .route("/health", get(public::health))
        .route("/notes", get(public::notes_index))
        .route("/notes/{slug}", get(public::note_detail))
        .route("/tags/{tag}", get(public::tag_detail))
        .route("/categories/{category}", get(public::category_detail))
        .route("/search", get(public::search))
        .route("/account", get(account::account_page))
        .route("/account/register", post(account::register))
        .route("/account/login", post(account::login))
        .route("/account/logout", post(account::logout))
        .route(
            "/api/notes/{slug}/annotations",
            get(account::list_annotations).post(account::create_annotation),
        )
        .route(
            "/api/annotations/{id}",
            put(account::update_annotation).delete(account::delete_annotation),
        )
        .route("/admin", get(admin::dashboard))
        .route("/admin/login", get(admin::login_page).post(admin::login))
        .route("/admin/logout", post(admin::logout))
        .route("/admin/password", post(admin::change_password))
        .route(
            "/admin/users",
            get(admin::users_page).post(admin::create_public_user),
        )
        .route("/admin/users/{username}", get(admin::user_detail_page))
        .route(
            "/admin/users/{username}/update",
            post(admin::update_public_user),
        )
        .route(
            "/admin/users/{username}/delete",
            post(admin::delete_public_user),
        )
        .route("/admin/notes/new", get(admin::new_note_page))
        .route("/admin/notes/{slug}/edit", get(admin::edit_note_page))
        .route("/admin/notes/save", post(admin::save_note))
        .route(
            "/admin/upload/markdown",
            post(admin::upload_markdown).layer(DefaultBodyLimit::max(upload_limit_bytes)),
        )
        .route(
            "/admin/upload/asset",
            post(admin::upload_asset).layer(DefaultBodyLimit::max(upload_limit_bytes)),
        )
        .route("/admin/rebuild", post(admin::rebuild_site))
        .nest_service("/static", static_service)
        .nest_service("/assets", assets_service)
        .with_state(state)
}
