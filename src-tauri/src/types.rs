use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 存档信息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct Archive {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    #[ts(type = "number")]
    pub file_size: i64,
    pub checksum: String,
    #[ts(type = "number")]
    pub chunk_count: i64,
    pub note: String,
    pub tags: Vec<String>,
    pub parent_id: Option<String>,
    pub created_at: String,
}

/// 存档简要信息（用于列表展示）
pub type ArchiveInfo = Archive;

/// 已标记的重要版本
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct StarredArchive {
    pub archive: ArchiveInfo,
    pub star_id: String,
    pub label: String,
}

/// 目录恢复结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct RestoreDirectoryResult {
    pub restored_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}

/// 历史导出结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct ExportResult {
    pub output_path: String,
    pub archive_count: usize,
    #[ts(type = "number")]
    pub total_size: u64,
}

/// 差异结果
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct DiffResult {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

/// 差异块
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub changes: Vec<DiffChange>,
}

/// 差异变更
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct DiffChange {
    pub change_type: String,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

/// 差异统计
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
    pub unchanged: u32,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct Statistics {
    #[ts(type = "number")]
    pub total_archives: i64,
    #[ts(type = "number")]
    pub total_size: i64,
    #[ts(type = "number")]
    pub unique_files: i64,
    #[ts(type = "number")]
    pub total_chunks: i64,
    #[ts(type = "number")]
    pub storage_chunks: Option<i64>,
    #[ts(type = "number")]
    pub storage_bytes: Option<i64>,
}
