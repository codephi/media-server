use axum::{
    body::Body,
    extract::{Path, State},
    http::header,
    response::Response,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::controllers::AppState;
use crate::models::{fs, media, Result};

/// Force download of file
pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response> {
    let path = path.trim_matches('/');
    let full_path = fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;
    
    // Get filename for Content-Disposition
    let filename = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");
    
    // Encode filename for Content-Disposition
    let encoded_filename = utf8_percent_encode(filename, NON_ALPHANUMERIC).to_string();
    
    // Detect MIME type
    let (mime_type, _) = media::detect(&full_path);
    
    // Get file size
    let metadata = tokio::fs::metadata(&full_path).await?;
    let file_size = metadata.len();
    
    // Open file for streaming
    let file = File::open(&full_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let response = Response::builder()
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename*=UTF-8''{}", encoded_filename),
        )
        .body(body)
        .unwrap();
    
    Ok(response)
}