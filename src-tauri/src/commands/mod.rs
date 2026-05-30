use tauri::State;
use crate::db::Archive;
use crate::diff::DiffResult;
use crate::error::AppError;
use crate::AppState;

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
    state
        .service
        .restore_archive(&id, target_path.as_deref())
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
pub async fn delete_archive(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    state.service.delete_archive(&id)
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
