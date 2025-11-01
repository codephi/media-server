use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "media-serve",
    version,
    about = "A media file server with MVC architecture"
)]
pub struct Cli {
    /// Directory to serve
    #[arg(value_name = "BASE_DIR", value_hint = ValueHint::DirPath)]
    pub base_dir: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Host/IP to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Default thumbnail size (largest side in pixels)
    #[arg(long = "thumb-size", default_value_t = 320)]
    pub thumb_size: u32,

    /// Log level (error|warn|info|debug|trace)
    #[arg(long = "log-level", default_value = "info")]
    pub log_level: String,

    /// Show hidden files by default
    #[arg(long = "show-hidden")]
    pub show_hidden: bool,

    /// Enable file watching for auto-reload during development
    #[arg(long)]
    pub watch: bool,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub base_dir: PathBuf,
    pub base_dir_canonical: PathBuf,
    pub thumb_size: u32,
    pub show_hidden: bool,
    pub ffmpeg_available: bool,
    pub watch_enabled: bool,
}

impl AppConfig {
    pub fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        // Validate and canonicalize base directory
        if !cli.base_dir.exists() {
            anyhow::bail!("Directory does not exist: {}", cli.base_dir.display());
        }
        if !cli.base_dir.is_dir() {
            anyhow::bail!("Path is not a directory: {}", cli.base_dir.display());
        }

        let base_dir_canonical = cli.base_dir.canonicalize()?;

        // Check for ffmpeg availability
        let ffmpeg_available = std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !ffmpeg_available {
            tracing::warn!("ffmpeg not found - video thumbnails will not be generated");
        }

        Ok(Self {
            base_dir: cli.base_dir.clone(),
            base_dir_canonical,
            thumb_size: cli.thumb_size,
            show_hidden: cli.show_hidden,
            ffmpeg_available,
            watch_enabled: cli.watch,
        })
    }
}
