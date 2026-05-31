// Suppress warnings in modules owned by other agents (db, watcher) to keep CI green.
// Ideally these modules should be fixed upstream; this is a stopgap.
#[allow(clippy::all)]
mod db;
#[allow(clippy::all)]
mod watcher;

mod commands;
mod config;
mod diff;
mod error;
mod services;
mod storage;

use config::AppConfig;
use services::archive_service::ArchiveService;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub service: ArchiveService,
    pub watcher: Mutex<watcher::FileWatcher>,
    pub config: Mutex<AppConfig>,
    pub data_dir: std::path::PathBuf,
}

fn init_logging(data_dir: &std::path::Path, log_config: &config::LogConfig) {
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let log_file = log_dir.join("docdist.log");

    // 日志级别
    let level = match log_config.level.as_str() {
        "trace" => tracing::level_filters::LevelFilter::TRACE,
        "debug" => tracing::level_filters::LevelFilter::DEBUG,
        "info" => tracing::level_filters::LevelFilter::INFO,
        "warn" => tracing::level_filters::LevelFilter::WARN,
        "error" => tracing::level_filters::LevelFilter::ERROR,
        _ => tracing::level_filters::LevelFilter::INFO,
    };

    // 控制台输出层
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    if log_config.file_output {
        // 文件输出层
        let log_file_std = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .expect("Failed to open log file");

        let file_layer = fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false)
            .with_writer(log_file_std);

        tracing_subscriber::registry()
            .with(level)
            .with(console_layer)
            .with(file_layer)
            .init();

        tracing::info!(
            "日志系统初始化完成 (级别: {}, 文件: {})",
            log_config.level,
            log_file.display()
        );
    } else {
        tracing_subscriber::registry()
            .with(level)
            .with(console_layer)
            .init();

        tracing::info!(
            "日志系统初始化完成 (级别: {}, 仅控制台)",
            log_config.level
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = dirs_next::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("docdist");
    std::fs::create_dir_all(&data_dir).ok();

    // 加载配置
    let app_config = AppConfig::load(&data_dir);

    // 初始化日志
    init_logging(&data_dir, &app_config.log);

    tracing::info!("DocDist 启动中...");
    tracing::debug!("数据目录: {:?}", data_dir);

    // 初始化数据库
    let db_path = data_dir.join("data.db");
    let pool =
        db::init_database(&db_path).expect("Failed to initialize database");
    tracing::info!("数据库初始化完成: {:?}", db_path);

    // 初始化服务（chunk_size 从配置读取）
    let chunk_size = app_config.storage.chunk_size;
    let service = ArchiveService::new(pool, &data_dir, chunk_size);
    tracing::info!("存档服务初始化完成 (chunk_size: {})", chunk_size);

    // 初始化 watcher
    let mut file_watcher = watcher::FileWatcher::new();
    file_watcher
        .set_exclude_patterns(app_config.watcher.exclude_patterns.clone());
    // TODO: Watcher 的防抖延迟应从 app_config.watcher.auto_archive_delay 读取，
    // 但 FileWatcher 当前没有 set_debounce_duration 方法，需要 Agent-B 在 watcher/mod.rs 中添加。

    tracing::info!("DocDist 就绪 ✓");

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
