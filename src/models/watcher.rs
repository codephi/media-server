use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, Result as NotifyResult};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum WatchEvent {
    Reload,
}

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<NotifyResult<Event>>,
}

impl FileWatcher {
    pub fn new() -> anyhow::Result<(Self, broadcast::Sender<WatchEvent>)> {
        let (tx, rx) = mpsc::channel();
        let (broadcast_tx, _) = broadcast::channel(100);
        
        let mut watcher = notify::recommended_watcher(tx)?;
        
        // Watch templates directory
        if Path::new("templates").exists() {
            watcher.watch(Path::new("templates"), RecursiveMode::Recursive)?;
            tracing::info!("Watching templates/ directory for changes");
        }
        
        // Watch public directory for CSS/JS changes
        if Path::new("public").exists() {
            watcher.watch(Path::new("public"), RecursiveMode::Recursive)?;
            tracing::info!("Watching public/ directory for changes");
        }
        
        // Watch static assets if they exist
        if Path::new("static").exists() {
            watcher.watch(Path::new("static"), RecursiveMode::Recursive)?;
            tracing::info!("Watching static/ directory for changes");
        }
        
        let file_watcher = Self {
            _watcher: watcher,
            receiver: rx,
        };
        
        Ok((file_watcher, broadcast_tx))
    }
    
    pub async fn run(mut self, broadcast_tx: broadcast::Sender<WatchEvent>) {
        let mut debounce_timer = tokio::time::Instant::now();
        let debounce_duration = Duration::from_millis(100);
        
        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    if self.should_reload(&event) {
                        let now = tokio::time::Instant::now();
                        if now.duration_since(debounce_timer) > debounce_duration {
                            debounce_timer = now;
                            
                            tracing::info!("File change detected: {:?}", event.paths);
                            if let Err(e) = broadcast_tx.send(WatchEvent::Reload) {
                                tracing::error!("Failed to send reload event: {}", e);
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("Watch error: {}", e);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No events, sleep a bit
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    tracing::warn!("File watcher disconnected");
                    break;
                }
            }
        }
    }
    
    fn should_reload(&self, event: &Event) -> bool {
        // Only react to write/create/remove events
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {},
            _ => return false,
        }
        
        // Check if any path matches our criteria
        event.paths.iter().any(|path| {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                matches!(extension, "html" | "css" | "js" | "ts")
            } else {
                false
            }
        })
    }
}