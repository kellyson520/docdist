use crate::config::AppConfig;
use crate::db::Archive;
use crate::diff::types::EnhancedDiffResult;
use crate::diff::DiffResult;
use crate::error::AppError;
use crate::storage;
use crate::AppState;
use tauri::Manager;
use tauri::State;

// ==================== 存档管理 ====================

#[tauri::command]
pub async fn create_archive(
    state: State<'_, AppState>,
    path: String,
    note: Option<String>,
    tags: Option<Vec<String>>,
    parent_id: Option<String>,
) -> Result<Archive, AppError> {
    state.service.create_archive(
        &path,
        note.as_deref().unwrap_or(""),
        tags.unwrap_or_default(),
        parent_id,
    )
}

#[tauri::command]
pub async fn restore_archive(
    state: State<'_, AppState>,
    id: String,
    target_path: Option<String>,
) -> Result<(), AppError> {
    state.service.restore_archive(&id, target_path.as_deref())
}

#[tauri::command]
pub async fn list_archives(
    state: State<'_, AppState>,
    file_path: Option<String>,
    search: Option<String>,
) -> Result<Vec<Archive>, AppError> {
    state
        .service
        .list_archives(file_path.as_deref(), search.as_deref())
}

#[tauri::command]
pub async fn list_archives_paginated(
    state: State<'_, AppState>,
    file_path: Option<String>,
    search: Option<String>,
    page: u32,
    page_size: u32,
) -> Result<(Vec<Archive>, i64), AppError> {
    state.service.list_archives_paginated(
        file_path.as_deref(),
        search.as_deref(),
        page,
        page_size,
    )
}

#[tauri::command]
pub async fn delete_archive(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    state.service.delete_archive(&id)
}

#[tauri::command]
pub async fn delete_archives_batch(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<usize, AppError> {
    state.service.delete_archives_batch(&ids)
}

#[tauri::command]
pub async fn update_archive(
    state: State<'_, AppState>,
    id: String,
    note: String,
    tags: Vec<String>,
) -> Result<(), AppError> {
    state.service.update_archive(&id, &note, tags)
}

#[tauri::command]
pub async fn compare_archives(
    state: State<'_, AppState>,
    id1: String,
    id2: String,
) -> Result<DiffResult, AppError> {
    state.service.compare_archives(&id1, &id2)
}

/// 增强差异对比
#[tauri::command]
pub async fn compare_archives_enhanced(
    state: State<'_, AppState>,
    id1: String,
    id2: String,
) -> Result<EnhancedDiffResult, AppError> {
    state.service.compare_archives_enhanced(&id1, &id2)
}

#[tauri::command]
pub async fn get_timeline(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<Archive>, AppError> {
    state.service.get_timeline(&path)
}

#[tauri::command]
pub async fn get_children(
    state: State<'_, AppState>,
    parent_id: String,
) -> Result<Vec<Archive>, AppError> {
    state.service.get_children(&parent_id)
}

#[tauri::command]
pub async fn get_archive_tree(
    state: State<'_, AppState>,
    file_path: Option<String>,
) -> Result<Vec<Archive>, AppError> {
    state.service.get_archive_tree(file_path.as_deref())
}

#[tauri::command]
pub async fn get_statistics(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    state.service.get_statistics()
}

// ==================== Watcher 控制 ====================

#[tauri::command]
pub async fn start_watcher(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<(), AppError> {
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());

    // 从配置读取防抖延迟
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    watcher.set_debounce_duration(std::time::Duration::from_secs(
        config.watcher.auto_archive_delay,
    ));
    drop(config);

    // 设置自动存档回调：通过 Tauri 事件通知前端
    let handle = app_handle.clone();
    watcher.set_auto_archive_callback(std::sync::Arc::new(
        move |path: String| {
            // 通过 Tauri 事件通知前端，让前端调用 create_archive
            let _ = handle.emit_all(
                "auto-archive-request",
                serde_json::json!({ "path": path }),
            );
        },
    ));

    watcher.start(paths, Some(app_handle))
}

#[tauri::command]
pub async fn stop_watcher(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.stop();
    Ok(())
}

#[tauri::command]
pub async fn get_watcher_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    let watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    Ok(serde_json::json!({
        "running": watcher.is_running(),
        "paths": watcher.get_watched(),
    }))
}

#[tauri::command]
pub async fn add_watcher_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.add_path(path)
}

#[tauri::command]
pub async fn remove_watcher_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.remove_path(&path)
}

#[tauri::command]
pub async fn set_watcher_exclude_patterns(
    state: State<'_, AppState>,
    patterns: Vec<String>,
) -> Result<(), AppError> {
    let watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.set_exclude_patterns(patterns);
    Ok(())
}

// ==================== 存储管理 ====================

#[tauri::command]
pub async fn cleanup_orphan_chunks(
    state: State<'_, AppState>,
) -> Result<storage::CleanupStats, AppError> {
    state.service.cleanup_orphan_chunks()
}

#[tauri::command]
pub async fn verify_chunks(
    state: State<'_, AppState>,
) -> Result<Vec<String>, AppError> {
    state.service.verify_chunks()
}

// ==================== 配置管理 ====================

// ==================== 日志管理 ====================

#[tauri::command]
pub async fn read_log_file(
    state: State<'_, AppState>,
    lines: Option<usize>,
) -> Result<Vec<String>, String> {
    let max_lines = lines.unwrap_or(100);
    let log_path = state.data_dir.join("logs").join("docdist.log");

    if !log_path.exists() {
        return Ok(vec![]);
    }

    // 尾部读取：仅读取文件末尾约 max_lines * 200 字节，避免大文件 OOM
    let file_size = std::fs::metadata(&log_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);
    let read_limit = max_lines * 200; // 假设每行平均约 200 字节
    let skip = file_size.saturating_sub(read_limit);

    use std::io::{BufRead, Seek, SeekFrom};
    let file = std::fs::File::open(&log_path)
        .map_err(|e| format!("读取日志文件失败: {}", e))?;
    let mut reader = std::io::BufReader::new(file);
    if skip > 0 {
        reader
            .seek(SeekFrom::Start(skip as u64))
            .map_err(|e| format!("读取日志文件失败: {}", e))?;
        // 跳过被截断的第一行
        let mut discard = String::new();
        let _ = reader.read_line(&mut discard);
    }
    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let total = all_lines.len();
    let start = total.saturating_sub(max_lines);

    Ok(all_lines[start..].to_vec())
}

#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<AppConfig, AppError> {
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    Ok(config.clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), AppError> {
    // 配置验证：关键字段不能为零
    if new_config.storage.chunk_size == 0 {
        return Err(AppError::Other("chunk_size 不能为 0".to_string()));
    }

    // 先保存到磁盘，失败则不更新内存状态，保证一致性
    new_config.save(&state.data_dir)?;

    let mut config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    *config = new_config;

    // 如果 watcher 配置变了，应用到 watcher
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.set_exclude_patterns(config.watcher.exclude_patterns.clone());
    watcher.set_debounce_duration(std::time::Duration::from_secs(
        config.watcher.auto_archive_delay,
    ));

    Ok(())
}
