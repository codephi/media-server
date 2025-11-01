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
    tracing::info!("Starting upload to path: '{}'", path);

    // Resolve target directory
    let target_dir = fs::canonicalize_in_base(&state.config.base_dir_canonical, path)?;

    if !target_dir.is_dir() {
        return Err(AppError::BadRequest(
            "Target is not a directory".to_string(),
        ));
    }

    tracing::info!("Target directory: {}", target_dir.display());

    let mut uploaded_count = 0;
    let mut field_count = 0;

    // Process each field in the multipart form
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                field_count += 1;
                tracing::debug!("Processing field #{}", field_count);

                // Get field name
                let field_name = field.name().unwrap_or("unknown");
                tracing::debug!("Field name: '{}'", field_name);

                // Get content type if available
                if let Some(content_type) = field.content_type() {
                    tracing::debug!("Content type: {}", content_type);
                }

                // Skip non-file fields
                if field_name != "files" {
                    tracing::debug!("Skipping non-file field: {}", field_name);
                    continue;
                }

                // Get filename
                let filename = match field.file_name() {
                    Some(name) => {
                        tracing::debug!("Original filename: '{}'", name);
                        sanitize_filename(name)
                    }
                    None => {
                        tracing::warn!("Field 'files' has no filename, skipping");
                        continue;
                    }
                };

                if filename.is_empty() {
                    tracing::warn!("Empty filename after sanitization, skipping");
                    continue;
                }

                tracing::info!("Uploading file: '{}'", filename);

                // Find available filename
                let file_path = fs::next_available_name(&target_dir, &filename);
                tracing::debug!("Target file path: {}", file_path.display());

                // Create file and collect all data first
                let mut data = Vec::new();
                let mut chunk_count = 0;

                // Read all chunks into memory first
                let mut field = field;
                loop {
                    match field.chunk().await {
                        Ok(Some(chunk)) => {
                            chunk_count += 1;
                            data.extend_from_slice(&chunk);
                            if chunk_count % 100 == 0 {
                                tracing::debug!(
                                    "Read {} chunks, total size: {} bytes",
                                    chunk_count,
                                    data.len()
                                );
                            }
                        }
                        Ok(None) => {
                            tracing::debug!(
                                "Finished reading file, {} chunks, {} bytes total",
                                chunk_count,
                                data.len()
                            );
                            break;
                        }
                        Err(e) => {
                            tracing::error!("Failed to read chunk {}: {}", chunk_count + 1, e);
                            return Err(AppError::BadRequest(format!(
                                "Failed to read file data: {}",
                                e
                            )));
                        }
                    }
                }

                // Write all data to file at once
                match File::create(&file_path).await {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(&data).await {
                            tracing::error!(
                                "Failed to write to file {}: {}",
                                file_path.display(),
                                e
                            );
                            return Err(AppError::Internal(format!("Failed to write file: {}", e)));
                        }

                        if let Err(e) = file.flush().await {
                            tracing::error!("Failed to flush file {}: {}", file_path.display(), e);
                            return Err(AppError::Internal(format!("Failed to save file: {}", e)));
                        }

                        uploaded_count += 1;
                        tracing::info!(
                            "Successfully uploaded file: {} ({} bytes)",
                            file_path.display(),
                            data.len()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to create file {}: {}", file_path.display(), e);
                        return Err(AppError::Internal(format!("Failed to create file: {}", e)));
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("No more fields to process");
                break;
            }
            Err(e) => {
                tracing::error!(
                    "Failed to get next field after processing {} fields: {}",
                    field_count,
                    e
                );
                return Err(AppError::BadRequest(format!(
                    "Failed to process form field: {}",
                    e
                )));
            }
        }
    }

    tracing::info!("Processed {} fields total", field_count);

    if uploaded_count == 0 {
        tracing::warn!("No files were uploaded from {} fields", field_count);
        return Err(AppError::BadRequest("No files were uploaded".to_string()));
    }

    tracing::info!("Upload completed: {} file(s) uploaded", uploaded_count);

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
