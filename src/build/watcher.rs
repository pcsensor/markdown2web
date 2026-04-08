use std::{sync::Arc, time::Duration};

use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::{app::AppState, error::AppResult};

pub async fn spawn_watcher(state: AppState) -> AppResult<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |event| {
            let _ = tx.send(event);
        },
        NotifyConfig::default(),
    )?;
    watcher.watch(&state.config.notes_dir, RecursiveMode::Recursive)?;
    watcher.watch(&state.config.assets_dir, RecursiveMode::Recursive)?;

    let watcher = Arc::new(std::sync::Mutex::new(Some(watcher)));
    state.set_watcher_handle(watcher.clone()).await;

    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(800)).await;
            while rx.try_recv().is_ok() {}
            let _ = state
                .build_service
                .rebuild("filesystem watcher")
                .await
                .map_err(|err| eprintln!("watcher rebuild failed: {err}"));
        }
    });
    Ok(())
}
