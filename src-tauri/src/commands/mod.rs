use crate::config::AppConfig;
use crate::db::Archive;
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
    let mut config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    *config = new_config.clone();

    // 保存到磁盘
    config.save(&state.data_dir)?;

    // 如果 watcher 配置变了，应用到 watcher
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.set_exclude_patterns(config.watcher.exclude_patterns.clone());

    Ok(())
}
