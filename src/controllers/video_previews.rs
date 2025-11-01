use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::controllers::AppState;
use crate::models::{video_previews, Result};

#[derive(Deserialize)]
pub struct PreviewQuery {
    time: Option<f64>,
}

/// Serve video preview thumbnails
pub async fn video_preview(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<PreviewQuery>,
) -> Result<impl IntoResponse> {
    let path = path.trim_matches('/');

    // Get or build preview info
    let preview_info = match video_previews::get_or_build_previews(
        &state.config.base_dir_canonical,
        path,
        state.config.ffmpeg_available,
    )
    .await?
    {
        Some(info) => info,
        None => {
            return Err(crate::models::AppError::NotFound(
                "Preview não disponível".to_string(),
            ))
        }
    };

    // If no specific time requested, return preview info as JSON
    let target_time = match query.time {
        Some(time) => time,
        None => {
            let json = serde_json::to_string(&preview_info).map_err(|e| {
                crate::models::AppError::Internal(format!("Erro ao serializar JSON: {}", e))
            })?;
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json))
                .unwrap());
        }
    };

    // Get thumbnail path for specific time
    let thumb_path = match video_previews::get_preview_thumbnail_path(
        &state.config.base_dir_canonical,
        path,
        target_time,
        &preview_info,
    ) {
        Some(path) => path,
        None => {
            return Err(crate::models::AppError::NotFound(
                "Miniatura não encontrada".to_string(),
            ))
        }
    };

    // Read and serve thumbnail
    match tokio::fs::read(&thumb_path).await {
        Ok(data) => Ok(Response::builder()
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(Body::from(data))
            .unwrap()),
        Err(_) => Err(crate::models::AppError::NotFound(
            "Arquivo de miniatura não encontrado".to_string(),
        )),
    }
}
