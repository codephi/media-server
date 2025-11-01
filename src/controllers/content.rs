use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

use crate::controllers::AppState;
use crate::models::{fs, media, Result};

/// Parse Range header
fn parse_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_header.starts_with("bytes=") {
        return None;
    }
    
    let range = &range_header[6..];
    let parts: Vec<&str> = range.split('-').collect();
    
    if parts.len() != 2 {
        return None;
    }
    
    let start = if parts[0].is_empty() {
        // "-500" means last 500 bytes
        let suffix_len: u64 = parts[1].parse().ok()?;
        file_size.saturating_sub(suffix_len)
    } else {
        parts[0].parse().ok()?
    };
    
    let end = if parts[1].is_empty() {
        // "500-" means from byte 500 to end
        file_size - 1
    } else {
        let end: u64 = parts[1].parse().ok()?;
        end.min(file_size - 1)
    };
    
    if start > end || start >= file_size {
        return None;
    }
    
    Some((start, end))
}

/// Serve file content with Range support
pub async fn raw_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response> {
    let path = path.trim_matches('/');
    let full_path = fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;
    
    // Get file metadata
    let metadata = tokio::fs::metadata(&full_path).await?;
    let file_size = metadata.len();
    
    // Detect MIME type
    let (mime_type, _) = media::detect(&full_path);
    
    // Check for Range header
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| parse_range(v, file_size));
    
    let mut response = Response::builder();
    response = response.header(header::ACCEPT_RANGES, "bytes");
    response = response.header(header::CONTENT_TYPE, mime_type);
    
    if let Some((start, end)) = range {
        // Partial content
        let content_length = end - start + 1;
        
        response = response.status(StatusCode::PARTIAL_CONTENT);
        response = response.header(header::CONTENT_LENGTH, content_length.to_string());
        response = response.header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, file_size),
        );
        
        // Open file and seek to start
        let mut file = File::open(&full_path).await?;
        use tokio::io::AsyncSeekExt;
        file.seek(tokio::io::SeekFrom::Start(start)).await?;
        
        // Create limited stream
        let limited = file.take(content_length);
        let stream = ReaderStream::new(limited);
        let body = Body::from_stream(stream);
        
        Ok(response.body(body).unwrap())
    } else {
        // Full content
        response = response.status(StatusCode::OK);
        response = response.header(header::CONTENT_LENGTH, file_size.to_string());
        
        let file = File::open(&full_path).await?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);
        
        Ok(response.body(body).unwrap())
    }
}