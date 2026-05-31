#![allow(warnings)]
#![allow(clippy::all)]
mod commands;
mod config;
mod db;
mod diff;
mod error;
mod services;
mod storage;
mod watcher;

use config::AppConfig;
use services::archive_service::ArchiveService;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub service: ArchiveService,
    pub watcher: Mutex<watcher::FileWatcher>,
    pub config: Mutex<AppConfig>,
    pub data_dir: std::path::PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let data_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("docdist");
    std::fs::create_dir_all(&data_dir).ok();

    // 加载配置
    let app_config = AppConfig::load(&data_dir);

    // 初始化数据库
    let db_path = data_dir.join("data.db");
    let pool =
        db::init_database(&db_path).expect("Failed to initialize database");

    // 初始化服务
    let service = ArchiveService::new(pool, &data_dir);

    // 初始化 watcher，应用配置中的排除模式
    let mut file_watcher = watcher::FileWatcher::new();
    file_watcher
        .set_exclude_patterns(app_config.watcher.exclude_patterns.clone());

    tauri::Builder::default()
        .manage(AppState {
            service,
            watcher: Mutex::new(file_watcher),
            config: Mutex::new(app_config),
            data_dir: data_dir.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            // 存档管理
            commands::create_archive,
            commands::restore_archive,
            commands::list_archives,
            commands::list_archives_paginated,
            commands::delete_archive,
            commands::delete_archives_batch,
            commands::update_archive,
            commands::compare_archives,
            commands::get_timeline,
            commands::get_children,
            commands::get_statistics,
            // Watcher 控制
            commands::start_watcher,
            commands::stop_watcher,
            commands::get_watcher_status,
            commands::add_watcher_path,
            commands::remove_watcher_path,
            commands::set_watcher_exclude_patterns,
            // 存储管理
            commands::cleanup_orphan_chunks,
            commands::verify_chunks,
            // 配置管理
            commands::get_config,
            commands::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
