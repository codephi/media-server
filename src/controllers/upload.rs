use axum::{
    extract::{Multipart, Path, State},
    response::{IntoResponse, Redirect},
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::controllers::AppState;
use crate::models::{fs, AppError, Result};

/// Handle file uploads
pub async fn upload(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let path = path.trim_matches('/');
    
    // Resolve target directory
    let target_dir = fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;
    
    if !target_dir.is_dir() {
        return Err(AppError::BadRequest("Target is not a directory".to_string()));
    }
    
    // Process each file
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to process upload: {}", e))
    })? {
        // Get filename
        let name = field.name().unwrap_or("file");
        if name != "files" {
            continue;
        }
        
        let filename = field
            .file_name()
            .map(|n| sanitize_filename(n))
            .ok_or_else(|| AppError::BadRequest("Missing filename".to_string()))?;
        
        if filename.is_empty() {
            return Err(AppError::BadRequest("Invalid filename".to_string()));
        }
        
        // Find available filename
        let file_path = fs::next_available_name(&target_dir, &filename);
        
        // Create file and write data
        let mut file = File::create(&file_path).await?;
        
        while let Some(chunk) = field.chunk().await.map_err(|e| {
            AppError::BadRequest(format!("Failed to read chunk: {}", e))
        })? {
            file.write_all(&chunk).await?;
        }
        
        file.flush().await?;
        
        tracing::info!("Uploaded file: {}", file_path.display());
    }
    
    // Redirect back to the directory
    let redirect_url = format!("/browse/{}", path);
    Ok(Redirect::to(&redirect_url))
}

/// Sanitize filename for safe storage
fn sanitize_filename(name: &str) -> String {
    // Remove path separators and null bytes
    let cleaned: String = name
        .chars()
        .filter(|c| !matches!(*c, '/' | '\\' | '\0'))
        .collect();
    
    // Limit length
    if cleaned.len() > 255 {
        let mut truncated = cleaned.chars().take(200).collect::<String>();
        if let Some(ext_pos) = cleaned.rfind('.') {
            if let Some(ext) = cleaned.get(ext_pos..) {
                if ext.len() <= 10 {
                    truncated.push_str(ext);
                }
            }
        }
        truncated
    } else {
        cleaned
    }
}