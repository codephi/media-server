mod controllers;
mod models;
mod views;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = models::Cli::parse();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(format!("media_serve={},tower_http=warn", cli.log_level))
        .init();

    // Create app config
    let config = models::AppConfig::from_cli(&cli)?;

    tracing::info!(
        "Starting media-serve for directory: {}",
        config.base_dir.display()
    );
    tracing::info!("ffmpeg available: {}", config.ffmpeg_available);

    // Setup file watcher if enabled
    let state = if config.watch_enabled {
        tracing::info!("Watch mode enabled - monitoring file changes");
        
        match models::watcher::FileWatcher::new() {
            Ok((file_watcher, watch_sender)) => {
                let state = controllers::AppState::new_with_watcher(config, watch_sender.clone());
                
                // Start the file watcher in background
                tokio::spawn(async move {
                    file_watcher.run(watch_sender).await;
                });
                
                state
            }
            Err(e) => {
                tracing::error!("Failed to start file watcher: {}", e);
                controllers::AppState::new(config)
            }
        }
    } else {
        controllers::AppState::new(config)
    };

    // Build router
    let app = Router::new()
        .route("/", get(controllers::browse::root_redirect))
        .route("/browse/", get(controllers::browse::browse))
        .route("/browse/*path", get(controllers::browse::browse))
        .route("/file/*path", get(controllers::file::file_page))
        .route("/download/*path", get(controllers::download::download))
        .route("/content/*path", get(controllers::content::raw_content))
        .route("/thumbs/*path", get(controllers::thumbs::thumb))
        .route(
            "/video-previews/*path",
            get(controllers::video_previews::video_preview),
        )
        .route("/search/", get(controllers::search::search_page))
        .route("/search", get(controllers::search::search))
        .route("/upload/", post(controllers::upload::upload))
        .route("/upload/*path", post(controllers::upload::upload))
        .nest_service("/static", ServeDir::new("public"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", cli.bind, cli.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
