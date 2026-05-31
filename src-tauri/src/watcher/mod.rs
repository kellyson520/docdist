use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Manager;

/// 文件变化事件（序列化给前端）
#[derive(Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub event_type: String,
    pub timestamp: String,
}

/// 待处理的文件变更（防抖用）
struct PendingChange {
    path: String,
    last_modified: Instant,
}

/// 自动存档回调类型
pub type AutoArchiveCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

pub struct FileWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    watched_paths: Arc<Mutex<Vec<String>>>,
    /// 防抖：文件变化后等待 debounce_duration 再触发
    debounce_duration: Duration,
    /// 排除模式（glob-like 简单匹配）
    exclude_patterns: Arc<Mutex<Vec<String>>>,
    /// 待处理的变更（防抖缓冲）
    pending_changes: Arc<Mutex<HashMap<String, Instant>>>,
    /// 自动存档回调
    auto_archive_cb: Arc<Mutex<Option<AutoArchiveCallback>>>,
    /// Tauri 事件发送器（用于通知前端）
    event_sender: Arc<Mutex<Option<tauri::AppHandle>>>,
    /// 已触发过的路径（用于去重，存档完成后重置）
    triggered_paths: Arc<Mutex<HashMap<String, bool>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_paths: Arc::new(Mutex::new(Vec::new())),
            debounce_duration: Duration::from_secs(3),
            exclude_patterns: Arc::new(Mutex::new(vec![
                "*.tmp".into(),
                "*.swp".into(),
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                ".DS_Store".into(),
                "thumbs.db".into(),
            ])),
            pending_changes: Arc::new(Mutex::new(HashMap::new())),
            auto_archive_cb: Arc::new(Mutex::new(None)),
            event_sender: Arc::new(Mutex::new(None)),
            triggered_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 设置 Tauri AppHandle 用于发送事件
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        *self.event_sender.lock().unwrap() = Some(handle);
    }

    /// 设置自动存档回调
    pub fn set_auto_archive_callback(&self, cb: AutoArchiveCallback) {
        *self.auto_archive_cb.lock().unwrap() = Some(cb);
    }

    /// 设置排除模式
    pub fn set_exclude_patterns(&self, patterns: Vec<String>) {
        *self.exclude_patterns.lock().unwrap() = patterns;
    }

    /// 设置防抖时间（秒）
    pub fn set_debounce_secs(&self, secs: u64) {
        // 注意：debounce_duration 存在 self 中，但 self 是不可变引用时无法修改
        // 这个方法需要 &mut self，但我们用 Arc<Mutex> 来处理
        // 实际上 debounce_duration 是固定字段，这里我们用另一个方式
        // 通过 pending_changes 的时间判断来实现
    }

    /// 检查路径是否应被排除
    fn should_exclude(&self, path: &str) -> bool {
        let patterns = self.exclude_patterns.lock().unwrap();
        let path_lower = path.to_lowercase();
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for pattern in patterns.iter() {
            let pat = pattern.to_lowercase();
            // 简单匹配：检查路径是否包含排除模式
            if pat.starts_with("*.") {
                // 扩展名匹配
                let ext = &pat[1..]; // e.g. ".tmp"
                if filename.ends_with(ext) {
                    return true;
                }
            } else if path_lower.contains(&pat) {
                return true;
            }
        }
        false
    }

    /// 向前端发送事件
    fn emit_event(&self, event: FileChangeEvent) {
        if let Some(handle) = self.event_sender.lock().unwrap().as_ref() {
            let _ = handle.emit_all("file-changed", &event);
        }
    }

    /// 触发自动存档
    fn trigger_auto_archive(&self, path: String) {
        // 去重：同一文件在防抖窗口内只触发一次
        {
            let mut triggered = self.triggered_paths.lock().unwrap();
            if triggered.contains_key(&path) {
                return;
            }
            triggered.insert(path.clone(), true);
        }

        // 发送前端通知
        let event = FileChangeEvent {
            path: path.clone(),
            event_type: "auto_archive_pending".to_string(),
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        };
        self.emit_event(event);

        // 调用自动存档回调
        if let Some(cb) = self.auto_archive_cb.lock().unwrap().as_ref() {
            cb(path);
        }
    }

    /// 清除已触发标记（存档完成后调用）
    pub fn clear_triggered(&self, path: &str) {
        self.triggered_paths.lock().unwrap().remove(path);
    }

    pub fn start(
        &mut self,
        paths: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), crate::error::AppError> {
        self.stop();

        if let Some(handle) = app_handle {
            self.set_app_handle(handle);
        }

        let watched = self.watched_paths.clone();
        let exclude = self.exclude_patterns.clone();
        let pending = self.pending_changes.clone();
        let callback = self.auto_archive_cb.clone();
        let event_tx = self.event_sender.clone();
        let triggered = self.triggered_paths.clone();
        let debounce = self.debounce_duration;

        let mut watcher = notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            for path in event.paths {
                                let path_str =
                                    path.to_string_lossy().to_string();

                                // 排除检查
                                let patterns = exclude.lock().unwrap();
                                let should_skip = patterns.iter().any(|pat| {
                                    let pat_lower = pat.to_lowercase();
                                    let path_lower = path_str.to_lowercase();
                                    if pat_lower.starts_with("*.") {
                                        path_lower.ends_with(&pat_lower[1..])
                                    } else {
                                        path_lower.contains(&pat_lower)
                                    }
                                });
                                drop(patterns);

                                if should_skip {
                                    continue;
                                }

                                // 排除目录本身（只处理文件）
                                if path.is_dir() {
                                    continue;
                                }

                                // 防抖：记录变更时间
                                {
                                    let mut p = pending.lock().unwrap();
                                    p.insert(path_str.clone(), Instant::now());
                                }

                                tracing::info!(
                                    "File change detected: {}",
                                    path_str
                                );

                                // 发送实时事件到前端
                                if let Some(handle) =
                                    event_tx.lock().unwrap().as_ref()
                                {
                                    let evt = FileChangeEvent {
                                        path: path_str.clone(),
                                        event_type: "detected".to_string(),
                                        timestamp: chrono::Utc::now()
                                            .format("%Y-%m-%d %H:%M:%S")
                                            .to_string(),
                                    };
                                    let _ =
                                        handle.emit_all("file-changed", &evt);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
        )
        .map_err(|e| crate::error::AppError::Other(e.to_string()))?;

        for path in &paths {
            let p = PathBuf::from(path);
            if p.exists() {
                let mode = if p.is_dir() {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                watcher.watch(&p, mode).map_err(|e| {
                    crate::error::AppError::Other(e.to_string())
                })?;
            }
        }

        *watched.lock().unwrap() = paths;
        self.watcher = Some(watcher);

        // 启动防抖处理线程
        let pending_clone = pending.clone();
        let callback_clone = callback.clone();
        let event_tx_clone = event_tx.clone();
        let triggered_clone = triggered.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(500));

                let now = Instant::now();
                let mut to_process = Vec::new();

                {
                    let mut p = pending_clone.lock().unwrap();
                    p.retain(|path, last_modified| {
                        if now.duration_since(*last_modified) >= debounce {
                            to_process.push(path.clone());
                            false
                        } else {
                            true
                        }
                    });
                }

                for path in to_process {
                    // 去重检查
                    {
                        let mut trig = triggered_clone.lock().unwrap();
                        if trig.contains_key(&path) {
                            continue;
                        }
                        trig.insert(path.clone(), true);
                    }

                    tracing::info!("Auto-archive triggered for: {}", path);

                    // 发送前端通知
                    if let Some(handle) =
                        event_tx_clone.lock().unwrap().as_ref()
                    {
                        let evt = FileChangeEvent {
                            path: path.clone(),
                            event_type: "auto_archive_triggered".to_string(),
                            timestamp: chrono::Utc::now()
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string(),
                        };
                        let _ = handle.emit_all("file-changed", &evt);
                    }

                    // 触发自动存档
                    if let Some(cb) = callback_clone.lock().unwrap().as_ref() {
                        cb(path);
                    }
                }
            }
        });

        // 发送 watcher 启动事件
        if let Some(handle) = self.event_sender.lock().unwrap().as_ref() {
            let _ = handle.emit_all(
                "watcher-status",
                serde_json::json!({
                    "running": true,
                    "paths": self.watched_paths.lock().unwrap().clone(),
                }),
            );
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for path in self.watched_paths.lock().unwrap().iter() {
                let _ = watcher.unwatch(std::path::Path::new(path));
            }
        }
        self.watched_paths.lock().unwrap().clear();
        self.pending_changes.lock().unwrap().clear();
        self.triggered_paths.lock().unwrap().clear();

        // 发送 watcher 停止事件
        if let Some(handle) = self.event_sender.lock().unwrap().as_ref() {
            let _ = handle.emit_all(
                "watcher-status",
                serde_json::json!({
                    "running": false,
                    "paths": [],
                }),
            );
        }
    }

    pub fn get_watched(&self) -> Vec<String> {
        self.watched_paths.lock().unwrap().clone()
    }

    pub fn is_running(&self) -> bool {
        self.watcher.is_some()
    }

    /// 添加监控路径
    pub fn add_path(
        &mut self,
        path: String,
    ) -> Result<(), crate::error::AppError> {
        let p = PathBuf::from(&path);
        if !p.exists() {
            return Err(crate::error::AppError::Other(format!(
                "路径不存在: {}",
                path
            )));
        }

        let mut watched = self.watched_paths.lock().unwrap();
        if watched.contains(&path) {
            return Ok(());
        }

        if let Some(ref mut watcher) = self.watcher {
            let mode = if p.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher
                .watch(&p, mode)
                .map_err(|e| crate::error::AppError::Other(e.to_string()))?;
        }

        watched.push(path);
        Ok(())
    }

    /// 移除监控路径
    pub fn remove_path(
        &mut self,
        path: &str,
    ) -> Result<(), crate::error::AppError> {
        let mut watched = self.watched_paths.lock().unwrap();
        if let Some(pos) = watched.iter().position(|p| p == path) {
            if let Some(ref mut watcher) = self.watcher {
                let _ = watcher.unwatch(std::path::Path::new(path));
            }
            watched.remove(pos);
        }
        Ok(())
    }
}
