use serde::{Deserialize, Serialize};

use super::DiffResult;

/// 差异摘要结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub stats: DiffStats,
    pub changes: Vec<ChangeSummary>,
    pub change_distribution: ChangeDistribution,
    pub affected_regions: Vec<AffectedRegion>,
    pub ai_summary: Option<String>,
}

/// 摘要统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub unchanged: usize,
    pub total: usize,
}

/// 单个变更摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub id: u32,
    pub change_type: ChangeType,
    pub location: ChangeLocation,
    pub description: String,
    pub snippet: Option<String>,
    pub line_count: usize,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Addition,
    Deletion,
    Modification,
    Rename,
    Move,
    FormatChange,
    EncodingChange,
    Replacement,
}

/// 变更位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLocation {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_col: Option<u32>,
    pub end_col: Option<u32>,
    pub region_description: Option<String>,
}

/// 变更分布统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDistribution {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub moves: usize,
    pub renames: usize,
}

/// 受影响的区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedRegion {
    pub name: String,
    pub region_type: RegionType,
    pub change_type: ChangeType,
    pub change_lines: usize,
    pub start_line: u32,
    pub end_line: u32,
}

/// 区域类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionType {
    Function,
    Class,
    Method,
    File,
    Directory,
    Section,
    Paragraph,
    Page,
    Layer,
    Block,
}

/// 文件类型检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Text {
        encoding: String,
        line_ending: String,
    },
    Binary {
        mime_type: String,
        size: u64,
    },
    PDF {
        page_count: u32,
        has_images: bool,
    },
    CAD {
        format: String,
        layer_count: u32,
        entity_count: u32,
    },
    Image {
        width: u32,
        height: u32,
        format: String,
    },
    Office {
        format: String,
        page_count: Option<u32>,
    },
}

/// 完整的差异对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDiffResult {
    pub diff_result: DiffResult,
    pub summary: DiffSummary,
    pub file_type: FileType,
    pub preview: Option<ContentPreview>,
}

/// 内容预览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPreview {
    pub old_preview: String,
    pub new_preview: String,
    pub preview_lines: usize,
}

/// 二进制差异结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDiffResult {
    pub identical: bool,
    pub size_change: i64,
    pub old_size: u64,
    pub new_size: u64,
    pub old_hash: String,
    pub new_hash: String,
    pub similarity: f64,
    pub summary: String,
}
