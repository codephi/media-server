pub mod browse;
pub mod content;
pub mod download;
pub mod file;
pub mod thumbs;
pub mod upload;
pub mod video_previews;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::models::AppConfig,
}

impl AppState {
    pub fn new(config: crate::models::AppConfig) -> Arc<Self> {
        Arc::new(Self { config })
    }
}
