use crate::config::AppConfig;
use crate::db::{self, Archive};
use crate::diff::types::EnhancedDiffResult;
use crate::diff::DiffResult;
use crate::error::AppError;
use crate::storage;
use crate::types::{ExportResult, RestoreDirectoryResult, StarredArchive};
use crate::AppState;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri::State;

const MAX_EXCLUDE_PATTERNS: usize = 128;
const MAX_PATTERN_LEN: usize = 256;
const MAX_WATCH_DIRS: usize = 128;
const MAX_CHUNK_SIZE: usize = 256 * 1024 * 1024; // 256MB
const MAX_WATCHER_DELAY_SECS: u64 = 3600;
const MAX_LOG_FILE_SIZE_MB: u64 = 1024;
const MAX_LOG_RETENTION_DAYS: u32 = 3650;
const MAX_EXTERNAL_TOOL_LEN: usize = 4096;

const ALLOWED_DIFF_TOOLS: &[&str] = &[
    "meld",
    "kdiff3",
    "code",
    "code-insiders",
    "diffmerge",
    "bcompare",
    "Beyond Compare",
    "tkdiff",
    "xxdiff",
    "kompare",
    "diffuse",
];

/// Directories where absolute-path diff tools are allowed to reside.
/// Only tools in these directories may be used when not in the whitelist.
const ALLOWED_TOOL_DIRS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",
    "/usr/sbin",
    "/opt",
    "/snap/bin",
    "/Applications", // macOS
];

/// Validate that the diff tool is either in the whitelist or an absolute path
/// in an allowed directory to an existing executable file.
///
/// Security: This prevents arbitrary program execution by restricting
/// external tools to a known whitelist or tools installed in standard
/// system directories.
fn validate_diff_tool(tool: &std::ffi::OsStr) -> Result<(), AppError> {
    let tool_path = std::path::Path::new(tool);
    let tool_str = tool.to_string_lossy();

    // Reject null bytes
    if tool_str.contains('\0') {
        return Err(AppError::Other(
            "外部对比工具路径包含非法空字节".to_string(),
        ));
    }

    // Reject shell metacharacters
    if tool_str.contains(';')
        || tool_str.contains('|')
        || tool_str.contains('&')
        || tool_str.contains('$')
        || tool_str.contains('`')
        || tool_str.contains('\n')
        || tool_str.contains('\r')
    {
        return Err(AppError::Other(
            "外部对比工具路径包含非法字符".to_string(),
        ));
    }

    // Check if it's a known safe tool name (no path separators)
    if !tool_str.contains('/')
        && !tool_str.contains('\\')
        && ALLOWED_DIFF_TOOLS.iter().any(|&t| t == tool_str.as_ref())
    {
        return Ok(());
    }

    // For absolute paths: require the tool to exist AND reside in an allowed directory
    if tool_path.is_absolute() && tool_path.exists() {
        // Canonicalize to prevent path traversal via ../
        let canon = std::fs::canonicalize(tool_path).map_err(|e| {
            AppError::Other(format!("工具路径解析失败: {}", e))
        })?;

        // Check that the canonicalized path is in an allowed directory
        let canon_str = canon.to_string_lossy();
        let in_allowed_dir = ALLOWED_TOOL_DIRS.iter().any(|dir| {
            canon_str.starts_with(dir)
                && canon_str.as_bytes().get(dir.len()) == Some(&b'/')
        });

        if !in_allowed_dir {
            return Err(AppError::Other(format!(
                "外部对比工具 '{}' 不在允许的目录中 (允许: {:?})",
                canon.display(),
                ALLOWED_TOOL_DIRS
            )));
        }

        // Verify it's a regular file (not a device, socket, etc.)
        let metadata = std::fs::metadata(&canon).map_err(|e| {
            AppError::Other(format!("无法读取工具元数据: {}", e))
        })?;
        if !metadata.is_file() {
            return Err(AppError::Other(
                "外部对比工具路径不是普通文件".to_string(),
            ));
        }

        return Ok(());
    }

    Err(AppError::Other(format!(
        "外部对比工具 '{}' 不在允许列表中，且不是允许目录中的绝对路径",
        tool_str
    )))
}

fn validate_exclude_patterns(patterns: &[String]) -> Result<(), AppError> {
    if patterns.len() > MAX_EXCLUDE_PATTERNS {
        return Err(AppError::Other(format!(
            "排除规则不能超过 {} 条",
            MAX_EXCLUDE_PATTERNS
        )));
    }
    for pattern in patterns {
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("排除规则不能为空".to_string()));
        }
        if trimmed.len() > MAX_PATTERN_LEN {
            return Err(AppError::Other(format!(
                "排除规则长度不能超过 {} 字符",
                MAX_PATTERN_LEN
            )));
        }
    }
    Ok(())
}

fn validate_config(config: &AppConfig) -> Result<(), AppError> {
    if config.storage.chunk_size == 0 {
        return Err(AppError::Other("chunk_size 不能为 0".to_string()));
    }
    if config.storage.chunk_size > MAX_CHUNK_SIZE {
        return Err(AppError::Other(format!(
            "chunk_size 不能超过 {}MB",
            MAX_CHUNK_SIZE / 1024 / 1024
        )));
    }
    if config.watcher.auto_archive_delay > MAX_WATCHER_DELAY_SECS {
        return Err(AppError::Other(format!(
            "自动存档延迟不能超过 {} 秒",
            MAX_WATCHER_DELAY_SECS
        )));
    }
    if config.watcher.max_file_size > 0
        && config.watcher.min_file_size > config.watcher.max_file_size
    {
        return Err(AppError::Other(
            "最小文件大小不能大于最大文件大小".to_string(),
        ));
    }
    if config.watcher.watch_dirs.len() > MAX_WATCH_DIRS {
        return Err(AppError::Other(format!(
            "监控目录不能超过 {} 个",
            MAX_WATCH_DIRS
        )));
    }
    validate_exclude_patterns(&config.watcher.exclude_patterns)?;

    match config.log.level.as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => {}
        _ => {
            return Err(AppError::Other(
                "日志级别必须是 trace/debug/info/warn/error".to_string(),
            ));
        }
    }
    if config.log.max_file_size_mb == 0
        || config.log.max_file_size_mb > MAX_LOG_FILE_SIZE_MB
    {
        return Err(AppError::Other(format!(
            "日志文件大小必须在 1..={}MB",
            MAX_LOG_FILE_SIZE_MB
        )));
    }
    if config.log.retention_days > MAX_LOG_RETENTION_DAYS {
        return Err(AppError::Other(format!(
            "日志保留天数不能超过 {} 天",
            MAX_LOG_RETENTION_DAYS
        )));
    }
    if let Some(tool) = &config.external_diff_tool {
        if tool.to_string_lossy().trim().len() > MAX_EXTERNAL_TOOL_LEN {
            return Err(AppError::Other(format!(
                "外部对比工具路径不能超过 {} 字符",
                MAX_EXTERNAL_TOOL_LEN
            )));
        }
    }
    Ok(())
}

fn chunk_path_for_hash(
    chunks_dir: &Path,
    hash: &str,
) -> Result<PathBuf, AppError> {
    if hash.len() < 2 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Other(format!("无效的 chunk hash: {}", hash)));
    }
    Ok(chunks_dir.join(&hash[..2]).join(hash))
}

fn safe_archive_entry_name(archive_id: &str, index: usize) -> String {
    let sanitized: String = archive_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_');
    let fallback = "archive";
    let stem = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    let limited: String = stem.chars().take(128).collect();
    format!("archives/{:06}_{}.bin", index + 1, limited)
}

fn safe_temp_archive_file_name(prefix: &str, archive: &Archive) -> String {
    let sanitized: String = archive
        .file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = sanitized.trim_matches('_');
    let stem = if stem.is_empty() { "archive" } else { stem };
    let limited: String = stem.chars().take(160).collect();
    format!(
        "{}-{}-{}",
        prefix,
        &archive.id[..8.min(archive.id.len())],
        limited
    )
}

fn configured_external_diff_tool(
    config: &AppConfig,
    override_tool: Option<String>,
) -> Result<std::ffi::OsString, AppError> {
    if let Some(tool) = override_tool {
        let trimmed = tool.trim();
        if !trimmed.is_empty() {
            if trimmed.len() > MAX_EXTERNAL_TOOL_LEN {
                return Err(AppError::Other(format!(
                    "外部对比工具路径不能超过 {} 字符",
                    MAX_EXTERNAL_TOOL_LEN
                )));
            }
            return Ok(std::ffi::OsString::from(trimmed));
        }
    }

    config
        .external_diff_tool
        .as_ref()
        .and_then(|p| {
            let raw = p.as_os_str();
            if raw.is_empty() {
                None
            } else {
                Some(raw.to_os_string())
            }
        })
        .ok_or_else(|| {
            AppError::Other("请先在设置中配置外部对比工具路径".to_string())
        })
}

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
pub async fn open_external_diff(
    state: State<'_, AppState>,
    id1: String,
    id2: String,
    tool_path: Option<String>,
) -> Result<serde_json::Value, AppError> {
    let tool = {
        let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        configured_external_diff_tool(&config, tool_path)?
    };

    // Validate tool against whitelist/directory restrictions BEFORE any use
    validate_diff_tool(&tool)?;

    let archive1 = db::get_archive(state.service.db(), &id1)?
        .ok_or_else(|| AppError::Other("存档1不存在".to_string()))?;
    let archive2 = db::get_archive(state.service.db(), &id2)?
        .ok_or_else(|| AppError::Other("存档2不存在".to_string()))?;

    let temp_dir =
        std::env::temp_dir()
            .join("docdist-external-diff")
            .join(format!(
                "{}-{}",
                chrono::Utc::now().format("%Y%m%d%H%M%S"),
                uuid::Uuid::new_v4()
            ));
    std::fs::create_dir_all(&temp_dir)?;

    let left_path =
        temp_dir.join(safe_temp_archive_file_name("left", &archive1));
    let right_path =
        temp_dir.join(safe_temp_archive_file_name("right", &archive2));

    state
        .service
        .restore_archive(&archive1.id, left_path.to_str())?;
    state
        .service
        .restore_archive(&archive2.id, right_path.to_str())?;

    std::process::Command::new(&tool)
        .arg(&left_path)
        .arg(&right_path)
        .current_dir(&temp_dir)
        .spawn()
        .map_err(|e| {
            AppError::Other(format!(
                "启动外部对比工具失败: {} ({})",
                std::path::PathBuf::from(&tool).display(),
                e
            ))
        })?;

    Ok(serde_json::json!({
        "tool": std::path::PathBuf::from(&tool).to_string_lossy(),
        "left_path": left_path.to_string_lossy(),
        "right_path": right_path.to_string_lossy(),
    }))
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
    let (debounce_secs, min_file_size, max_file_size) = {
        let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        (
            config.watcher.auto_archive_delay,
            config.watcher.min_file_size,
            config.watcher.max_file_size,
        )
    };

    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher
        .set_debounce_duration(std::time::Duration::from_secs(debounce_secs));
    watcher.set_file_size_range(min_file_size, max_file_size);

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
    validate_exclude_patterns(&patterns)?;

    // 先更新 watcher（锁 watcher）
    {
        let watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
        watcher.set_exclude_patterns(patterns.clone());
    }
    // 再持久化到配置（锁 config，避免 ABBA 死锁）
    {
        let mut config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.watcher.exclude_patterns = patterns;
        config.save(&state.data_dir)?;
    }
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
) -> Result<Vec<String>, AppError> {
    let max_lines = lines.unwrap_or(100).min(10000); // Cap at 10000 lines
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
        .map_err(|e| AppError::Other(format!("读取日志文件失败: {}", e)))?;
    let mut reader = std::io::BufReader::new(file);
    if skip > 0 {
        reader
            .seek(SeekFrom::Start(skip as u64))
            .map_err(|e| AppError::Other(format!("读取日志文件失败: {}", e)))?;
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
    validate_config(&new_config)?;

    // 先保存到磁盘，失败则不更新内存状态，保证一致性
    new_config.save(&state.data_dir)?;

    // 提取 watcher 配置副本，然后释放 config 锁再锁 watcher，避免交叉持锁
    let (exclude_patterns, debounce_secs, min_file_size, max_file_size) = {
        let mut config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        *config = new_config;
        (
            config.watcher.exclude_patterns.clone(),
            config.watcher.auto_archive_delay,
            config.watcher.min_file_size,
            config.watcher.max_file_size,
        )
    };
    // config 锁已释放，安全获取 watcher 锁
    let mut watcher = state.watcher.lock().unwrap_or_else(|e| e.into_inner());
    watcher.set_exclude_patterns(exclude_patterns);
    watcher
        .set_debounce_duration(std::time::Duration::from_secs(debounce_secs));
    watcher.set_file_size_range(min_file_size, max_file_size);

    Ok(())
}

// ==================== 版本管理 ====================

/// 标记一个存档为重要版本
#[tauri::command]
pub async fn star_archive(
    state: State<'_, AppState>,
    archive_id: String,
    label: String,
) -> Result<(), AppError> {
    db::star_archive(state.service.db(), &archive_id, &label)?;
    Ok(())
}

/// 取消标记
#[tauri::command]
pub async fn unstar_archive(
    state: State<'_, AppState>,
    archive_id: String,
) -> Result<(), AppError> {
    db::unstar_archive(state.service.db(), &archive_id)?;
    Ok(())
}

/// 获取所有标记的版本
#[tauri::command]
pub async fn get_starred_archives(
    state: State<'_, AppState>,
) -> Result<Vec<StarredArchive>, AppError> {
    let results = db::get_starred_archives(state.service.db())?;
    Ok(results
        .into_iter()
        .map(|(archive, star_id, label)| StarredArchive {
            archive: crate::types::Archive {
                id: archive.id,
                file_path: archive.file_path,
                file_name: archive.file_name,
                file_size: archive.file_size,
                checksum: archive.checksum,
                chunk_count: archive.chunk_count,
                note: archive.note,
                tags: archive.tags,
                parent_id: archive.parent_id,
                created_at: archive.created_at,
            },
            star_id,
            label,
        })
        .collect())
}

/// 按路径模式搜索存档
#[tauri::command]
pub async fn search_archives_by_path(
    state: State<'_, AppState>,
    pattern: String,
) -> Result<Vec<Archive>, AppError> {
    db::get_archives_by_path_pattern(state.service.db(), &pattern)
}

/// 获取单个文件的完整版本历史（按时间排序）
#[tauri::command]
pub async fn get_file_history(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Vec<Archive>, AppError> {
    db::get_archives_by_file_path(state.service.db(), &file_path)
}

/// 目录级恢复：恢复某目录下所有文件到指定时间点之前的最新版本
#[tauri::command]
pub async fn restore_directory(
    state: State<'_, AppState>,
    dir_path: String,
    before_timestamp: String,
    output_dir: String,
) -> Result<RestoreDirectoryResult, AppError> {
    // 获取该目录下在指定时间之前的所有存档
    let archives = db::get_archives_by_dir_before(
        state.service.db(),
        &dir_path,
        &before_timestamp,
    )?;

    // 按文件路径分组，取每个文件最新的存档
    use std::collections::HashMap;
    let mut latest_per_file: HashMap<String, &Archive> = HashMap::new();
    for archive in &archives {
        latest_per_file
            .entry(archive.file_path.clone())
            .and_modify(|e| {
                if archive.created_at > e.created_at {
                    *e = archive;
                }
            })
            .or_insert(archive);
    }

    let mut restored_count = 0usize;
    let mut skipped_count = 0usize;
    let mut errors = Vec::new();

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_canonical = std::fs::canonicalize(&output_dir)?;

    for archive in latest_per_file.values() {
        let output_path =
            std::path::PathBuf::from(&output_dir).join(&archive.file_name);

        // 防止 zip-slip 路径遍历攻击
        match output_path.parent() {
            Some(parent) => {
                if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
                    if !canonical_parent.starts_with(&output_dir_canonical) {
                        errors.push(format!(
                            "{}: 路径安全检查失败",
                            archive.file_path
                        ));
                        skipped_count += 1;
                        continue;
                    }
                }
            }
            None => {
                errors.push(format!("{}: 无效的输出路径", archive.file_path));
                skipped_count += 1;
                continue;
            }
        }

        match state.service.restore_archive(
            &archive.id,
            Some(output_path.to_str().unwrap_or("")),
        ) {
            Ok(()) => restored_count += 1,
            Err(e) => {
                errors.push(format!("{}: {}", archive.file_path, e));
                skipped_count += 1;
            }
        }
    }

    Ok(RestoreDirectoryResult {
        restored_count,
        skipped_count,
        errors,
    })
}

/// 导出历史为ZIP包
#[tauri::command]
pub async fn export_history(
    state: State<'_, AppState>,
    file_path: Option<String>,
    output_dir: String,
) -> Result<ExportResult, AppError> {
    // 获取要导出的存档列表
    let archives = if let Some(ref fp) = file_path {
        db::get_archives_by_file_path(state.service.db(), fp)?
    } else {
        db::get_all_archives(state.service.db(), None, None)?
    };

    if archives.is_empty() {
        return Err(AppError::Other("没有找到可导出的存档".to_string()));
    }

    // 构建 ZIP 文件
    std::fs::create_dir_all(&output_dir)?;
    let output_path = std::path::PathBuf::from(&output_dir).join(format!(
        "history_export_{}.zip",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));

    let zip_file = std::fs::File::create(&output_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    // 写入 manifest.json
    let manifest = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "archive_count": archives.len(),
        "file_filter": file_path,
        "archives": archives.iter().map(|a| serde_json::json!({
            "id": a.id,
            "file_path": a.file_path,
            "file_name": a.file_name,
            "file_size": a.file_size,
            "checksum": a.checksum,
            "note": a.note,
            "tags": a.tags,
            "created_at": a.created_at,
        })).collect::<Vec<_>>(),
    });
    zip.start_file("manifest.json", options)?;
    use std::io::Write;
    zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;

    // 写入各存档的 chunk 文件
    let chunks_dir = state.service.chunks_dir();
    for (index, archive) in archives.iter().enumerate() {
        let chunk_hashes =
            db::get_archive_chunk_hashes(state.service.db(), &archive.id)?;
        let filename = safe_archive_entry_name(&archive.id, index);
        zip.start_file(&filename, options)?;
        for hash in &chunk_hashes {
            let chunk_path = chunk_path_for_hash(chunks_dir, hash)?;
            if !chunk_path.exists() {
                return Err(AppError::Other(format!("分块不存在: {}", hash)));
            }
            let mut chunk_file = std::fs::File::open(&chunk_path)?;
            std::io::copy(&mut chunk_file, &mut zip)?;
        }
    }

    zip.finish()?;
    let file_size = std::fs::metadata(&output_path)?.len();

    Ok(ExportResult {
        output_path: output_path.to_string_lossy().to_string(),
        archive_count: archives.len(),
        total_size: file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_safe_archive_entry_name_prevents_zip_slip() {
        let name = safe_archive_entry_name("../../evil/path", 0);
        assert_eq!(name, "archives/000001_evil_path.bin");
    }

    #[test]
    fn test_safe_archive_entry_name_has_fallback() {
        let name = safe_archive_entry_name("../../", 1);
        assert_eq!(name, "archives/000002_archive.bin");
    }

    #[test]
    fn test_chunk_path_for_hash_rejects_invalid_hash() {
        let result = chunk_path_for_hash(Path::new("/tmp/chunks"), "../bad");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_temp_archive_file_name_removes_path_separators() {
        let archive = Archive {
            id: "12345678-1234-1234-1234-123456789abc".to_string(),
            file_path: "/tmp/evil".to_string(),
            file_name: "../../evil.txt".to_string(),
            file_size: 1,
            checksum: "abc".to_string(),
            chunk_count: 1,
            note: String::new(),
            tags: vec![],
            parent_id: None,
            created_at: "2026-01-01 00:00:00.000".to_string(),
        };

        let name = safe_temp_archive_file_name("left", &archive);
        assert_eq!(name, "left-12345678-.._.._evil.txt");
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
    }

    #[test]
    fn test_configured_external_diff_tool_prefers_override() {
        let mut config = AppConfig::default();
        config.external_diff_tool = Some(PathBuf::from("meld"));

        let tool =
            configured_external_diff_tool(&config, Some("code".to_string()))
                .unwrap();

        assert_eq!(tool, std::ffi::OsString::from("code"));
    }

    #[test]
    fn test_configured_external_diff_tool_requires_config() {
        let config = AppConfig::default();
        let result = configured_external_diff_tool(&config, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_configured_external_diff_tool_rejects_long_override() {
        let config = AppConfig::default();
        let tool = "x".repeat(MAX_EXTERNAL_TOOL_LEN + 1);
        let result = configured_external_diff_tool(&config, Some(tool));

        assert!(result.is_err());
    }

    // ── Security tests for diff tool validation ─────────────────────

    #[test]
    fn test_validate_diff_tool_whitelist_meld() {
        assert!(validate_diff_tool(OsStr::new("meld")).is_ok());
    }

    #[test]
    fn test_validate_diff_tool_whitelist_kdiff3() {
        assert!(validate_diff_tool(OsStr::new("kdiff3")).is_ok());
    }

    #[test]
    fn test_validate_diff_tool_whitelist_vscode() {
        assert!(validate_diff_tool(OsStr::new("code")).is_ok());
    }

    #[test]
    fn test_validate_diff_tool_rejects_unknown_name() {
        assert!(validate_diff_tool(OsStr::new("evil-tool")).is_err());
    }

    #[test]
    fn test_validate_diff_tool_rejects_null_bytes() {
        assert!(
            validate_diff_tool(OsStr::new("meld\0; rm -rf /")).is_err()
        );
    }

    #[test]
    fn test_validate_diff_tool_rejects_shell_metacharacters() {
        assert!(validate_diff_tool(OsStr::new("meld; evil")).is_err());
        assert!(validate_diff_tool(OsStr::new("meld|evil")).is_err());
        assert!(validate_diff_tool(OsStr::new("meld&evil")).is_err());
        assert!(validate_diff_tool(OsStr::new("meld$evil")).is_err());
        assert!(validate_diff_tool(OsStr::new("meld`evil`")).is_err());
    }

    #[test]
    fn test_validate_diff_tool_rejects_unrestricted_absolute_path() {
        // /bin/sh exists but is not in an allowed tool directory
        // (it's in /bin, not in ALLOWED_TOOL_DIRS)
        let result = validate_diff_tool(OsStr::new("/bin/sh"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_diff_tool_rejects_dev_null() {
        // /dev/null exists but is not a regular file in an allowed dir
        let result = validate_diff_tool(OsStr::new("/dev/null"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_diff_tool_rejects_nonexistent_absolute_path() {
        let result =
            validate_diff_tool(OsStr::new("/nonexistent/tool/bin/meld"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_exclude_patterns_valid() {
        assert!(validate_exclude_patterns(&[
            "*.tmp".to_string(),
            "*.log".to_string()
        ])
        .is_ok());
    }

    #[test]
    fn test_validate_exclude_patterns_empty_pattern() {
        assert!(validate_exclude_patterns(&["".to_string()]).is_err());
    }

    #[test]
    fn test_validate_exclude_patterns_too_many() {
        let patterns: Vec<String> =
            (0..200).map(|i| format!("pattern_{}", i)).collect();
        assert!(validate_exclude_patterns(&patterns).is_err());
    }
}
