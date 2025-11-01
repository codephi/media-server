use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::controllers::AppState;
use crate::models::{media, thumbnails, Result};

#[derive(Debug, Deserialize)]
pub struct ThumbQuery {
    #[serde(default)]
    w: Option<u32>,
}

/// Serve or generate thumbnail
pub async fn thumb(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<ThumbQuery>,
) -> Result<Response> {
    let path = path.trim_matches('/');
    let width = query.w.unwrap_or(state.config.thumb_size);
    
    // Try to get or generate thumbnail
    let thumb_path = thumbnails::get_or_build(
        &state.config.base_dir_canonical,
        path,
        width,
        state.config.ffmpeg_available,
    )
    .await?;
    
    if let Some(thumb_path) = thumb_path {
        // Serve the thumbnail
        let metadata = tokio::fs::metadata(&thumb_path).await?;
        let file = File::open(&thumb_path).await?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);
        
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CONTENT_LENGTH, metadata.len().to_string())
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(body)
            .unwrap();
        
        Ok(response)
    } else {
        // No thumbnail available, redirect to appropriate icon
        let full_path = crate::models::fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;
        let (_, media_kind) = media::detect(&full_path);
        let icon_path = format!("/static/icons/{}", media_kind.icon_name());
        
        Ok(Redirect::temporary(&icon_path).into_response())
    }
}