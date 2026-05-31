export interface Archive {
  id: string;
  file_path: string;
  file_name: string;
  file_size: number;
  checksum: string;
  chunk_count: number;
  note: string;
  tags: string[];
  parent_id: string | null;
  created_at: string;
}

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
  change_type: 'add' | 'delete' | 'equal';
  content: string;
  old_line: number | null;
  new_line: number | null;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  unchanged: number;
}

export interface Statistics {
  total_archives: number;
  total_size: number;
  unique_files: number;
  total_chunks?: number;
  storage_chunks?: number;
  storage_bytes?: number;
}
