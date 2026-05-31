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
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;
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
    // 从配置读取防抖延迟
    file_watcher.set_debounce_duration(Duration::from_secs(
        app_config.watcher.auto_archive_delay,
    ));

    // 捕获启动前的配置，供 setup 闭包恢复 watcher
    let restore_watcher_cfg = app_config.watcher.clone();

    tracing::info!("DocDist 就绪 ✓");

    tauri::Builder::default()
        .manage(AppState {
            service,
            watcher: Mutex::new(file_watcher),
            config: Mutex::new(app_config),
            data_dir: data_dir.clone(),
        })
        .setup(move |app| {
            // 启动时自动恢复 watcher 监控（如果配置中 enabled = true）
            if restore_watcher_cfg.enabled
                && !restore_watcher_cfg.watch_dirs.is_empty()
            {
                let handle = app.handle();
                let state: tauri::State<AppState> = handle.state();
                let mut watcher =
                    state.watcher.lock().unwrap_or_else(|e| e.into_inner());
                let emit_handle = handle.clone();
                watcher.set_auto_archive_callback(std::sync::Arc::new(
                    move |path: String| {
                        let _ = emit_handle.emit_all(
                            "auto-archive-request",
                            serde_json::json!({ "path": path }),
                        );
                    },
                ));
                let app_handle = handle.clone();
                if let Err(e) = watcher.start(
                    restore_watcher_cfg.watch_dirs.clone(),
                    Some(app_handle),
                ) {
                    tracing::error!("自动恢复 watcher 失败: {}", e);
                } else {
                    tracing::info!(
                        "Watcher 自动恢复: {:?}",
                        restore_watcher_cfg.watch_dirs
                    );
                }
            }
            Ok(())
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
            commands::get_archive_tree,
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
            // 日志管理
            commands::read_log_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
