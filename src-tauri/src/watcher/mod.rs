use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
    /// 防抖线程停止信号
    debounce_stop: Arc<AtomicBool>,
    /// 防抖线程句柄
    debounce_handle: Option<std::thread::JoinHandle<()>>,
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
            debounce_stop: Arc::new(AtomicBool::new(false)),
            debounce_handle: None,
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

    /// 设置防抖持续时间
    pub fn set_debounce_duration(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    /// 设置自动存档回调
    #[allow(dead_code)]
    fn emit_event(&self, event: FileChangeEvent) {
        if let Some(handle) = self.event_sender.lock().unwrap().as_ref() {
            let _ = handle.emit_all("file-changed", &event);
        }
    }

    /// 触发自动存档
    #[allow(dead_code)]
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

        let debounce = self.debounce_duration;

        let watched = self.watched_paths.clone();
        let exclude = self.exclude_patterns.clone();
        let pending = self.pending_changes.clone();
        let event_tx = self.event_sender.clone();

        // 为 debounce 线程提前 clone
        let pending_debounce = self.pending_changes.clone();
        let callback_debounce = self.auto_archive_cb.clone();
        let event_tx_debounce = self.event_sender.clone();
        let triggered_debounce = self.triggered_paths.clone();

        let mut watcher = notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            for path in event.paths {
                                let path_str =
                                    path.to_string_lossy().to_string();

                                // 排除检查 — 按路径段匹配
                                let patterns = exclude.lock().unwrap();
                                let should_skip =
                                    is_path_excluded(&path_str, &patterns);
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

        // 启动防抖处理线程，使用 AtomicBool 控制退出
        let stop_signal = self.debounce_stop.clone();
        // 重置停止信号
        stop_signal.store(true, Ordering::SeqCst);

        let handle = std::thread::spawn(move || {
            while stop_signal.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(500));

                let now = Instant::now();
                let mut to_process = Vec::new();

                {
                    let mut p = pending_debounce.lock().unwrap();
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
                        let mut trig = triggered_debounce.lock().unwrap();
                        if trig.contains_key(&path) {
                            continue;
                        }
                        trig.insert(path.clone(), true);
                    }

                    tracing::info!("Auto-archive triggered for: {}", path);

                    // 发送前端通知
                    if let Some(handle) =
                        event_tx_debounce.lock().unwrap().as_ref()
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
                    if let Some(cb) = callback_debounce.lock().unwrap().as_ref()
                    {
                        cb(path);
                    }
                }
            }
        });
        self.debounce_handle = Some(handle);

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
        // 停止防抖线程
        self.debounce_stop.store(false, Ordering::SeqCst);
        if let Some(handle) = self.debounce_handle.take() {
            let _ = handle.join();
        }

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

/// 检查路径是否应被排除
/// 排除模式分为两类：
/// - `*.ext` 模式：匹配文件名后缀
/// - 目录名模式（如 `.git`、`node_modules`）：检查路径的每个部分是否匹配
fn is_path_excluded(path_str: &str, patterns: &[String]) -> bool {
    let path_lower = path_str.to_lowercase();
    for pat in patterns {
        let pat_lower = pat.to_lowercase();
        if pat_lower.starts_with("*.") {
            // 文件后缀匹配：*.tmp → 检查路径是否以 .tmp 结尾
            if path_lower.ends_with(&pat_lower[1..]) {
                return true;
            }
        } else {
            // 目录/文件名匹配：检查路径的每个部分是否完全匹配
            // 例如 pattern ".git" 匹配 "/foo/.git/bar" 和 "/foo/.git"
            for part in std::path::Path::new(path_str).components() {
                let part_str =
                    part.as_os_str().to_string_lossy().to_lowercase();
                if part_str == pat_lower {
                    return true;
                }
            }
        }
    }
    false
}
