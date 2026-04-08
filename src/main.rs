use std::sync::Arc;

use markdown2web::{app, build::watcher, config::AppConfig, store::sqlite::AppDatabase};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;
    config.ensure_directories()?;

    let db = Arc::new(AppDatabase::open(&config.database_path())?);
    db.initialize(&config.admin_username, &config.admin_password)?;

    let state = app::AppState::bootstrap(config.clone(), db).await?;
    if config.watch_enabled {
        watcher::spawn_watcher(state.clone()).await?;
    }

    let router = app::build_router(state);
    let bind = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&bind).await?;
    println!("markdown2web listening on http://{}", bind);
    axum::serve(listener, router).await?;
    Ok(())
}
