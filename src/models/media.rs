use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
    Audio,
    Pdf,
    Text,
    Archive,
    Other,
}

impl MediaKind {
    /// Get the icon filename for this media type
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Image => "image.svg",
            Self::Video => "video.svg",
            Self::Audio => "audio.svg",
            Self::Pdf => "pdf.svg",
            Self::Text => "text.svg",
            Self::Archive => "archive.svg",
            Self::Other => "file.svg",
        }
    }

    /// Check if this media type should show thumbnails
    pub fn has_thumbnail(&self) -> bool {
        matches!(self, Self::Image | Self::Video)
    }
}

/// Detect MIME type and media kind from file path
pub fn detect(path: &Path) -> (String, MediaKind) {
    // Try to detect from file content first
    if let Ok(kind) = infer::get_from_path(path) {
        if let Some(mime_type) = kind {
            let mime_str = mime_type.mime_type();
            let kind = media_kind_from_mime(mime_str);
            return (mime_str.to_string(), kind);
        }
    }

    // Fallback to guessing from extension
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mime_str = mime.to_string();
    let kind = media_kind_from_mime(&mime_str);

    (mime_str, kind)
}

/// Determine media kind from MIME type string
fn media_kind_from_mime(mime: &str) -> MediaKind {
    let mime = mime.to_lowercase();

    if mime.starts_with("image/") {
        MediaKind::Image
    } else if mime.starts_with("video/") {
        MediaKind::Video
    } else if mime.starts_with("audio/") {
        MediaKind::Audio
    } else if mime == "application/pdf" {
        MediaKind::Pdf
    } else if mime.starts_with("text/")
        || mime == "application/json"
        || mime == "application/xml"
        || mime == "application/javascript"
    {
        MediaKind::Text
    } else if mime == "application/zip"
        || mime == "application/x-rar-compressed"
        || mime == "application/x-tar"
        || mime == "application/gzip"
        || mime == "application/x-7z-compressed"
    {
        MediaKind::Archive
    } else {
        MediaKind::Other
    }
}
