// 增强差异对比类型定义

export interface DiffSummary {
  stats: DiffStats;
  changes: ChangeSummary[];
  change_distribution: ChangeDistribution;
  affected_regions: AffectedRegion[];
  ai_summary: string | null;
}

export interface ChangeSummary {
  id: number;
  change_type: 'Addition' | 'Deletion' | 'Modification' | 'Move' | 'Rename' | 'FormatChange' | 'EncodingChange' | 'Replacement';
  location: ChangeLocation;
  description: string;
  snippet: string | null;
  line_count: number;
}

export interface ChangeLocation {
  file_path: string;
  start_line: number;
  end_line: number;
  start_col: number | null;
  end_col: number | null;
  region_description: string | null;
}

export interface ChangeDistribution {
  additions: number;
  deletions: number;
  modifications: number;
  moves: number;
  renames: number;
}

export interface AffectedRegion {
  name: string;
  region_type: 'Function' | 'Class' | 'Method' | 'File' | 'Directory' | 'Section' | 'Paragraph' | 'Page' | 'Layer' | 'Block';
  change_type: string;
  change_lines: number;
  start_line: number;
  end_line: number;
}

export interface EnhancedDiffResult {
  diff_result: DiffResult;
  summary: DiffSummary;
  file_type: FileType;
  preview: ContentPreview | null;
}

export type FileType =
  | { type: 'Text'; encoding: string; line_ending: string }
  | { type: 'Binary'; mime_type: string; size: number }
  | { type: 'Pdf'; page_count: number; has_images: boolean }
  | { type: 'Cad'; format: string; layer_count: number; entity_count: number }
  | { type: 'Image'; width: number; height: number; format: string }
  | { type: 'Office'; format: string; page_count: number | null };

export interface ContentPreview {
  old_preview: string;
  new_preview: string;
  preview_lines: number;
}

// 已有类型引用（从 generated 复制，避免循环依赖）
export interface DiffResult {
  hunks: DiffHunk[];
  stats: DiffStats;
}

export interface DiffHunk {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  changes: DiffChange[];
}

export interface DiffChange {
  change_type: string;
  content: string;
  old_line: number | null;
  new_line: number | null;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  unchanged: number;
}
