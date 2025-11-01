use anyhow::{Context, Result};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub rel_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub name: String,
    pub rel_url: String,
}

/// Safely resolve a path within the base directory
pub fn canonicalize_in_base(base: &Path, rel_path: &str) -> Result<PathBuf> {
    // Normalize the relative path - remove leading slashes
    let rel_path = rel_path.trim_start_matches('/');

    // Join with base and canonicalize
    let full_path = if rel_path.is_empty() {
        base.to_path_buf()
    } else {
        base.join(rel_path)
    };

    let canonical = full_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", full_path.display()))?;

    // Ensure the canonical path starts with the base directory
    if !canonical.starts_with(base) {
        anyhow::bail!("Path traversal attempt detected");
    }

    Ok(canonical)
}

/// Get relative path from base directory
pub fn relative_from_base(base: &Path, abs_path: &Path) -> Result<String> {
    let rel_path = abs_path.strip_prefix(base).with_context(|| {
        format!(
            "Path {} is not under base {}",
            abs_path.display(),
            base.display()
        )
    })?;

    Ok(rel_path.to_string_lossy().into_owned())
}

/// List directory contents
pub fn list_dir(base: &Path, rel_path: &str, show_hidden: bool) -> Result<Vec<Item>> {
    let dir_path = canonicalize_in_base(base, rel_path)?;

    let mut items = Vec::new();

    for entry in std::fs::read_dir(&dir_path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip hidden files if not showing them
        if !show_hidden && is_hidden(&name) {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified = OffsetDateTime::from(metadata.modified()?);

        let full_path = entry.path();
        let item_rel_path = relative_from_base(base, &full_path)?;

        items.push(Item {
            name: name.clone(),
            rel_path: item_rel_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    // Sort: directories first, then files, both alphabetically
    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}

/// Get file information
pub fn file_info(base: &Path, rel_path: &str) -> Result<FileInfo> {
    let file_path = canonicalize_in_base(base, rel_path)?;

    if !file_path.is_file() {
        anyhow::bail!("Path is not a file: {}", file_path.display());
    }

    let metadata = std::fs::metadata(&file_path)?;
    let name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    Ok(FileInfo {
        name,
        size: metadata.len(),
        modified: OffsetDateTime::from(metadata.modified()?),
    })
}

/// Generate breadcrumbs from relative path
pub fn breadcrumbs(rel_path: &str) -> Vec<Breadcrumb> {
    let mut crumbs = vec![Breadcrumb {
        name: "Home".to_string(),
        rel_url: String::new(),
    }];

    if rel_path.is_empty() || rel_path == "/" {
        return crumbs;
    }

    let parts: Vec<&str> = rel_path.split('/').filter(|s| !s.is_empty()).collect();
    let mut accumulated = String::new();

    for part in parts {
        if !accumulated.is_empty() {
            accumulated.push('/');
        }
        accumulated.push_str(part);

        crumbs.push(Breadcrumb {
            name: part.to_string(),
            rel_url: accumulated.clone(),
        });
    }

    crumbs
}

/// Find next available filename in case of collision
pub fn next_available_name(dir: &Path, desired: &str) -> PathBuf {
    let mut candidate = dir.join(desired);

    if !candidate.exists() {
        return candidate;
    }

    // Split filename and extension
    let stem = Path::new(desired)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let extension = Path::new(desired)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_name = format!("{}({}){}", stem, counter, extension);
        candidate = dir.join(&new_name);

        if !candidate.exists() {
            break;
        }
        counter += 1;
    }

    candidate
}

/// Check if a filename is hidden (starts with dot)
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Encode a path for use in URLs
pub fn url_encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| percent_encode(segment.as_bytes(), NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Format file size for display
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format datetime for display
pub fn format_datetime(dt: &OffsetDateTime) -> String {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .unwrap_or_default();
    dt.format(&format).unwrap_or_else(|_| "Unknown".to_string())
}
