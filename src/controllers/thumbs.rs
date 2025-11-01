use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::controllers::AppState;
use crate::models::{thumbnails, Result};

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
        // No thumbnail available, return a simple gray placeholder
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <rect width="200" height="200" fill="rgb(30,41,59)"/>
  <rect x="20" y="20" width="160" height="160" fill="none" stroke="rgb(100,116,139)" stroke-width="2" stroke-dasharray="8,4"/>
  <text x="100" y="105" text-anchor="middle" fill="rgb(100,116,139)" font-family="sans-serif" font-size="12">Sem preview</text>
</svg>"#;

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(Body::from(svg_content))
            .unwrap();

        Ok(response)
    }
}
