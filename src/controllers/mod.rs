pub mod browse;
pub mod content;
pub mod dev;
pub mod download;
pub mod file;
pub mod search;
pub mod thumbs;
pub mod upload;
pub mod video_previews;

use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::models::AppConfig,
    pub watch_sender: Option<tokio::sync::broadcast::Sender<crate::models::watcher::WatchEvent>>,
}

impl AppState {
    pub fn new(config: crate::models::AppConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            watch_sender: None,
        })
    }

    pub fn new_with_watcher(
        config: crate::models::AppConfig,
        watch_sender: tokio::sync::broadcast::Sender<crate::models::watcher::WatchEvent>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            watch_sender: Some(watch_sender),
        })
    }
}
