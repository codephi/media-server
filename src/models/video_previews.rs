use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

type PreviewLocks = Arc<DashMap<String, Arc<Mutex<()>>>>;

lazy_static::lazy_static! {
    static ref PREVIEW_LOCKS: PreviewLocks = Arc::new(DashMap::new());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoPreviewInfo {
    pub duration: f64,
    pub interval: f64,
    pub thumbnails: Vec<VideoThumbnail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoThumbnail {
    pub time: f64,
    pub filename: String,
}

/// Get or build video preview thumbnails
pub async fn get_or_build_previews(
    base_dir: &Path,
    rel_path: &str,
    ffmpeg_available: bool,
) -> Result<Option<VideoPreviewInfo>> {
    if !ffmpeg_available {
        return Ok(None);
    }

    let abs_path = crate::models::fs::canonicalize_in_base(base_dir, rel_path)?;

    // Check if it's a video file
    let (_, media_kind) = crate::models::media::detect(&abs_path);
    if !matches!(media_kind, crate::models::media::MediaKind::Video) {
        return Ok(None);
    }

    // Generate cache key and directory
    let cache_key = blake3::hash(rel_path.as_bytes()).to_string();
    let cache_dir = abs_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Não foi possível obter o diretório pai"))?
        .join(".video-previews")
        .join(&cache_key);

    let info_file = cache_dir.join("info.json");

    // Check if preview info already exists
    if info_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&info_file).await {
            if let Ok(info) = serde_json::from_str::<VideoPreviewInfo>(&content) {
                return Ok(Some(info));
            }
        }
    }

    // Get lock for this specific video
    let lock = PREVIEW_LOCKS
        .entry(cache_key.clone())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();

    let _guard = lock.lock().await;

    // Check again after acquiring lock
    if info_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&info_file).await {
            if let Ok(info) = serde_json::from_str::<VideoPreviewInfo>(&content) {
                return Ok(Some(info));
            }
        }
    }

    // Create cache directory
    tokio::fs::create_dir_all(&cache_dir).await?;

    // Get video duration
    let duration = get_video_duration(&abs_path).await?;

    // Calculate interval (aim for ~100 thumbnails max)
    let max_thumbnails = 100;
    let interval = (duration / max_thumbnails as f64).max(1.0);

    // Generate thumbnails
    let mut thumbnails = Vec::new();
    let mut current_time = 0.0;

    while current_time < duration {
        let filename = format!("thumb_{:08.2}.jpg", current_time);
        let thumb_path = cache_dir.join(&filename);

        match generate_video_preview_thumbnail(&abs_path, &thumb_path, current_time).await {
            Ok(_) => {
                thumbnails.push(VideoThumbnail {
                    time: current_time,
                    filename,
                });
            }
            Err(e) => {
                tracing::warn!(
                    "Falha ao gerar miniatura para tempo {}: {}",
                    current_time,
                    e
                );
            }
        }

        current_time += interval;
    }

    let preview_info = VideoPreviewInfo {
        duration,
        interval,
        thumbnails,
    };

    // Save preview info
    let info_json = serde_json::to_string_pretty(&preview_info)?;
    tokio::fs::write(&info_file, info_json).await?;

    Ok(Some(preview_info))
}

/// Get video duration using ffprobe
async fn get_video_duration(video_path: &Path) -> Result<f64> {
    let output = tokio::process::Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("csv=p=0")
        .arg(video_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe falhou: {}", stderr);
    }

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let duration: f64 = duration_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Falha ao parsear duração: {}", e))?;

    Ok(duration)
}

/// Generate a single video thumbnail at specific time
async fn generate_video_preview_thumbnail(src: &Path, dest: &Path, time: f64) -> Result<()> {
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg(time.to_string())
        .arg("-i")
        .arg(src)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg("scale=160:90:force_original_aspect_ratio=decrease,pad=160:90:(ow-iw)/2:(oh-ih)/2")
        .arg("-q:v")
        .arg("5")
        .arg(dest)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg falhou: {}", stderr);
    }

    Ok(())
}

/// Get preview thumbnail path for a specific time
pub fn get_preview_thumbnail_path(
    base_dir: &Path,
    rel_path: &str,
    target_time: f64,
    preview_info: &VideoPreviewInfo,
) -> Option<PathBuf> {
    let abs_path = crate::models::fs::canonicalize_in_base(base_dir, rel_path).ok()?;
    let cache_key = blake3::hash(rel_path.as_bytes()).to_string();
    let cache_dir = abs_path.parent()?.join(".video-previews").join(&cache_key);

    // Find the closest thumbnail
    let mut closest_thumb: Option<&VideoThumbnail> = None;
    let mut min_diff = f64::INFINITY;

    for thumb in &preview_info.thumbnails {
        let diff = (thumb.time - target_time).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_thumb = Some(thumb);
        }
    }

    closest_thumb.map(|thumb| cache_dir.join(&thumb.filename))
}
