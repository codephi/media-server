use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use std::sync::Arc;
use std::time::Duration;

use crate::controllers::AppState;
use crate::models::watcher::WatchEvent;

pub async fn dev_reload_stream(
    State(state): State<Arc<AppState>>,
) -> crate::models::Result<Sse<impl Stream<Item = std::result::Result<Event, axum::Error>>>> {
    if !state.config.watch_enabled {
        return Err(crate::models::AppError::NotFound(
            "Watch mode not enabled".to_string(),
        ));
    }

    let mut watch_rx = state
        .watch_sender
        .as_ref()
        .ok_or_else(|| crate::models::AppError::Internal("Watch sender not available".to_string()))?
        .subscribe();

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                // Send periodic keep-alive
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    yield Ok(Event::default().comment("keep-alive"));
                }
                // Send reload event when file changes
                event = watch_rx.recv() => {
                    match event {
                        Ok(WatchEvent::Reload) => {
                            tracing::info!("Sending reload event to browser");
                            yield Ok(Event::default().event("reload").data("reload"));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Watch channel closed, ending stream");
                            break;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            tracing::warn!("Watch channel lagged, sending reload");
                            yield Ok(Event::default().event("reload").data("reload"));
                        }
                    }
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
