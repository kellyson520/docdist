use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;

#[derive(Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub event_type: String,
    pub timestamp: String,
}

pub struct FileWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    watched_paths: Arc<Mutex<Vec<String>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_paths: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self, paths: Vec<String>) -> Result<(), crate::error::AppError> {
        self.stop();

        let watched = self.watched_paths.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            tracing::info!("File changed: {:?}", path);
                        }
                    }
                    _ => {}
                }
            }
        }).map_err(|e| crate::error::AppError::Other(e.to_string()))?;

        for path in &paths {
            let p = PathBuf::from(path);
            if p.exists() {
                let mode = if p.is_dir() { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
                watcher.watch(&p, mode).map_err(|e| crate::error::AppError::Other(e.to_string()))?;
            }
        }

        *watched.lock().unwrap() = paths;
        self.watcher = Some(watcher);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for path in self.watched_paths.lock().unwrap().iter() {
                let _ = watcher.unwatch(std::path::Path::new(path));
            }
        }
        self.watched_paths.lock().unwrap().clear();
    }

    pub fn get_watched(&self) -> Vec<String> {
        self.watched_paths.lock().unwrap().clone()
    }
}
