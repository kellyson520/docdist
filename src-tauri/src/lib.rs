#![allow(clippy::all)]
mod commands;
mod db;
mod diff;
mod error;
mod services;
mod storage;
mod watcher;

use services::archive_service::ArchiveService;
use std::sync::Mutex;

pub struct AppState {
    pub service: ArchiveService,
    pub watcher: Mutex<watcher::FileWatcher>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let data_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("docdist");
    std::fs::create_dir_all(&data_dir).ok();

    let db_path = data_dir.join("data.db");
    let pool = db::init_database(&db_path)
        .expect("Failed to initialize database");

    let service = ArchiveService::new(pool, &data_dir);

    tauri::Builder::default()
        .manage(AppState {
            service,
            watcher: Mutex::new(
                watcher::FileWatcher::new(),
            ),
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_archive,
            commands::restore_archive,
            commands::list_archives,
            commands::delete_archive,
            commands::update_archive,
            commands::compare_archives,
            commands::get_timeline,
            commands::get_children,
            commands::get_statistics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
