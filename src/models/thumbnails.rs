use anyhow::Result;
use dashmap::DashMap;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

type ThumbLocks = Arc<DashMap<String, Arc<Mutex<()>>>>;

lazy_static::lazy_static! {
    static ref THUMB_LOCKS: ThumbLocks = Arc::new(DashMap::new());
}

/// Get or build a thumbnail for the given file
pub async fn get_or_build(
    base_dir: &Path,
    rel_path: &str,
    width: u32,
    ffmpeg_available: bool,
) -> Result<Option<PathBuf>> {
    let abs_path = crate::models::fs::canonicalize_in_base(base_dir, rel_path)?;
    
    // Check media type
    let (_, media_kind) = crate::models::media::detect(&abs_path);
    if !media_kind.has_thumbnail() {
        return Ok(None);
    }
    
    // Generate cache key and path
    let cache_key = format!("{}-w{}", blake3::hash(rel_path.as_bytes()), width);
    let cache_dir = base_dir.join(".media-serve").join("thumbs");
    let cache_path = cache_dir.join(format!("{}.jpg", cache_key));
    
    // Check if thumbnail already exists
    if cache_path.exists() {
        return Ok(Some(cache_path));
    }
    
    // Get lock for this specific thumbnail
    let lock = THUMB_LOCKS
        .entry(cache_key.clone())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();
    
    let _guard = lock.lock().await;
    
    // Check again after acquiring lock
    if cache_path.exists() {
        return Ok(Some(cache_path));
    }
    
    // Create cache directory if needed
    tokio::fs::create_dir_all(&cache_dir).await?;
    
    // Generate thumbnail based on media type
    match media_kind {
        crate::models::media::MediaKind::Image => {
            generate_image_thumbnail(&abs_path, &cache_path, width).await?;
            Ok(Some(cache_path))
        }
        crate::models::media::MediaKind::Video if ffmpeg_available => {
            match generate_video_thumbnail(&abs_path, &cache_path, width).await {
                Ok(_) => Ok(Some(cache_path)),
                Err(e) => {
                    tracing::warn!("Failed to generate video thumbnail: {}", e);
                    Ok(None)
                }
            }
        }
        _ => Ok(None),
    }
}

/// Generate thumbnail for an image file
async fn generate_image_thumbnail(src: &Path, dest: &Path, width: u32) -> Result<()> {
    let src = src.to_path_buf();
    let dest = dest.to_path_buf();
    
    tokio::task::spawn_blocking(move || {
        let img = image::open(&src)?;
        
        // Calculate new dimensions maintaining aspect ratio
        let (orig_width, orig_height) = img.dimensions();
        let ratio = width as f32 / orig_width.max(orig_height) as f32;
        
        if ratio < 1.0 {
            let new_width = (orig_width as f32 * ratio) as u32;
            let new_height = (orig_height as f32 * ratio) as u32;
            
            let resized = img.resize(
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
            
            resized.save_with_format(&dest, image::ImageFormat::Jpeg)?;
        } else {
            // Image is smaller than thumbnail size, just convert to JPEG
            img.save_with_format(&dest, image::ImageFormat::Jpeg)?;
        }
        
        Ok::<_, anyhow::Error>(())
    })
    .await??;
    
    Ok(())
}

/// Generate thumbnail for a video file using ffmpeg
async fn generate_video_thumbnail(src: &Path, dest: &Path, width: u32) -> Result<()> {
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg("1")
        .arg("-i")
        .arg(src)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg(format!("scale='min({},iw)':'min({},ih)':force_original_aspect_ratio=decrease", width, width))
        .arg("-q:v")
        .arg("2")
        .arg(dest)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr);
    }
    
    Ok(())
}