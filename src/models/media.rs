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
    /// Get the Iconify icon name for this media type
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Image => "heroicons:photo",
            Self::Video => "heroicons:play-circle",
            Self::Audio => "heroicons:musical-note",
            Self::Pdf => "heroicons:document-text",
            Self::Text => "heroicons:document",
            Self::Archive => "heroicons:archive-box",
            Self::Other => "heroicons:document",
        }
    }

    /// Check if this media type should show thumbnails
    pub fn has_thumbnail(&self) -> bool {
        matches!(self, Self::Image | Self::Video)
    }
}

/// Get folder icon name for Iconify
pub fn folder_icon() -> &'static str {
    "heroicons:folder"
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
