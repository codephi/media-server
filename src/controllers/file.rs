use askama_axum::Template;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::controllers::AppState;
use crate::models::{fs, media, Result};

#[derive(Template)]
#[template(path = "file.html")]
struct FileTemplate {
    file_info: fs::FileInfo,
    breadcrumbs: Vec<fs::Breadcrumb>,
    mime_type: String,
    media_kind: media::MediaKind,
    formatted_size: String,
    formatted_modified: String,
    encoded_path: String,
}

/// Show file page
pub async fn file_page(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse> {
    let path = path.trim_matches('/');
    
    let file_info = fs::file_info(&state.config.base_dir_canonical, path)?;
    let breadcrumbs = fs::breadcrumbs(path);
    
    let full_path = fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;
    let (mime_type, media_kind) = media::detect(&full_path);
    
    let template = FileTemplate {
        formatted_size: fs::format_size(file_info.size),
        formatted_modified: fs::format_datetime(&file_info.modified),
        encoded_path: fs::url_encode_path(path),
        file_info,
        breadcrumbs,
        mime_type,
        media_kind,
    };
    
    Ok(template)
}