use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// 是否启用文件监控
    pub enabled: bool,
    /// 监控的目录列表
    pub watch_dirs: Vec<String>,
    /// 排除的文件模式
    pub exclude_patterns: Vec<String>,
    /// 自动存档延迟（秒）
    pub auto_archive_delay: u64,
    /// 最小文件大小（字节）
    pub min_file_size: u64,
    /// 最大文件大小（字节）
    pub max_file_size: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            watch_dirs: Vec::new(),
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.swp".to_string(),
                ".git/*".to_string(),
                "node_modules/*".to_string(),
                "target/*".to_string(),
            ],
            auto_archive_delay: 60,
            min_file_size: 0,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 分块大小（字节）
    pub chunk_size: usize,
    /// 是否启用增量存储
    pub incremental: bool,
    /// 是否启用重复数据删除
    pub deduplication: bool,
    /// 存储路径
    pub storage_path: Option<PathBuf>,
    /// 保留版本数量（0=不限制）
    pub max_versions: u32,
    /// 自动清理天数（0=不清理）
    pub auto_cleanup_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            chunk_size: 4096,
            incremental: true,
            deduplication: true,
            storage_path: None,
            max_versions: 0,
            auto_cleanup_days: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 监控配置
    pub watcher: WatcherConfig,
    /// 存储配置
    pub storage: StorageConfig,
    /// UI 语言
    pub language: String,
    /// 主题
    pub theme: String,
    /// 是否开机自启
    pub auto_start: bool,
    /// 是否最小化到托盘
    pub minimize_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watcher: WatcherConfig::default(),
            storage: StorageConfig::default(),
            language: "zh-CN".to_string(),
            theme: "light".to_string(),
            auto_start: false,
            minimize_to_tray: true,
        }
    }
}

impl AppConfig {
    pub fn load(data_dir: &PathBuf) -> Self {
        let config_path = data_dir.join("config.json");
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(
        &self,
        data_dir: &PathBuf,
    ) -> Result<(), crate::error::AppError> {
        let config_path = data_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
