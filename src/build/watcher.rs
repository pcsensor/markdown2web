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
        loop {
            // 1. 阻塞等待第一个有效的文件变更事件
            let mut event_received = false;
            while let Some(result) = rx.recv().await {
                if let Ok(event) = result {
                    use notify::EventKind;
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            event_received = true;
                            break;
                        }
                        _ => continue,
                    }
                }
            }
            
            // 如果通道关闭，退出监听线程
            if !event_received {
                break;
            }

            // 2. 拖尾防抖：只要 800ms 内又有新的有效事件到来，就重置等待时间
            loop {
                match tokio::time::timeout(Duration::from_millis(800), rx.recv()).await {
                    Ok(Some(Ok(event))) => {
                        use notify::EventKind;
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                                // 收到新的有效事件，继续 inner loop 从而重置 800ms 超时
                                continue;
                            }
                            _ => continue, // 忽略无效事件（如访问事件）
                        }
                    }
                    Ok(Some(Err(_))) => continue, // 忽略 watcher 内部错误
                    Ok(None) => return, // 通道关闭，直接退出
                    Err(_) => {
                        // Timeout 触发，意味着已经有整整 800ms 没有新事件了
                        break;
                    }
                }
            }

            // 3. 执行重建
            let _ = state
                .build_service
                .rebuild("filesystem watcher")
                .await
                .map_err(|err| eprintln!("watcher rebuild failed: {err}"));
        }
    });
    Ok(())
}
