pub mod engine;
pub mod parsers; // 预留，T2 会创建 parsers/mod.rs
pub mod summary; // 预留，T4 会创建 summary.rs
pub mod types; // 预留，T4 会创建 engine.rs

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use tokio::sync::Semaphore;

/// 最大并发解析数
const MAX_PARSE_CONCURRENCY: usize = 4;

/// Number of unchanged context lines shown around each changed region.
const HUNK_CONTEXT_LINES: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub change_type: String,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
    pub unchanged: u32,
}

pub fn compute_diff(old_text: &str, new_text: &str) -> DiffResult {
    let diff = TextDiff::from_lines(old_text, new_text);

    let mut additions: u32 = 0;
    let mut deletions: u32 = 0;
    let mut unchanged: u32 = 0;
    let mut old_line: u32 = 1;
    let mut new_line: u32 = 1;
    let mut all_changes = Vec::new();
    let mut changed_indices = Vec::new();

    for change in diff.iter_all_changes() {
        let change_type = match change.tag() {
            ChangeTag::Equal => {
                unchanged += 1;
                "equal"
            }
            ChangeTag::Delete => {
                deletions += 1;
                "delete"
            }
            ChangeTag::Insert => {
                additions += 1;
                "add"
            }
        };

        let content = change.to_string();
        let content = content.trim_end_matches('\n');

        let old_ln = if change.tag() != ChangeTag::Insert {
            Some(old_line)
        } else {
            None
        };
        let new_ln = if change.tag() != ChangeTag::Delete {
            Some(new_line)
        } else {
            None
        };

        if change.tag() != ChangeTag::Equal {
            changed_indices.push(all_changes.len());
        }

        all_changes.push(DiffChange {
            change_type: change_type.to_string(),
            content: content.to_string(),
            old_line: old_ln,
            new_line: new_ln,
        });

        match change.tag() {
            ChangeTag::Equal | ChangeTag::Delete => old_line += 1,
            _ => {}
        }
        match change.tag() {
            ChangeTag::Equal | ChangeTag::Insert => new_line += 1,
            _ => {}
        }
    }

    let mut hunks = Vec::new();
    if !changed_indices.is_empty() {
        let grouped_ranges =
            group_changed_ranges(&changed_indices, all_changes.len());
        hunks = grouped_ranges
            .into_iter()
            .map(|(start, end)| build_hunk(&all_changes[start..end]))
            .collect();
    }

    DiffResult {
        hunks,
        stats: DiffStats {
            additions,
            deletions,
            unchanged,
        },
    }
}

fn group_changed_ranges(
    changed_indices: &[usize],
    total_changes: usize,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = changed_indices[0].saturating_sub(HUNK_CONTEXT_LINES);
    let mut end =
        (changed_indices[0] + HUNK_CONTEXT_LINES + 1).min(total_changes);

    for &idx in &changed_indices[1..] {
        let next_start = idx.saturating_sub(HUNK_CONTEXT_LINES);
        let next_end = (idx + HUNK_CONTEXT_LINES + 1).min(total_changes);
        if next_start <= end {
            end = end.max(next_end);
        } else {
            ranges.push((start, end));
            start = next_start;
            end = next_end;
        }
    }

    ranges.push((start, end));
    ranges
}

fn build_hunk(changes: &[DiffChange]) -> DiffHunk {
    let old_start = changes.iter().find_map(|c| c.old_line).unwrap_or(0);
    let new_start = changes.iter().find_map(|c| c.new_line).unwrap_or(0);
    let old_lines =
        changes.iter().filter(|c| c.old_line.is_some()).count() as u32;
    let new_lines =
        changes.iter().filter(|c| c.new_line.is_some()).count() as u32;

    DiffHunk {
        old_start,
        old_lines,
        new_start,
        new_lines,
        changes: changes.to_vec(),
    }
}

/// 单个文件解析结果
#[derive(Debug)]
pub struct ParsedFileResult {
    pub path: String,
    pub text: Result<String, String>,
}

/// 批量解析文件，使用 Semaphore 限制并发数（最大 MAX_PARSE_CONCURRENCY）。
///
/// 对于每个文件，通过 `parse_fn` 回调提取文本内容。
/// 所有文件在 Tokio 异步上下文中并发解析，但同时活跃的任务数不超过上限。
pub async fn batch_parse_files<F>(
    files: Vec<(String, Vec<u8>)>,
    parse_fn: F,
) -> Vec<ParsedFileResult>
where
    F: Fn(&[u8]) -> Result<String, String> + Send + Sync + 'static,
{
    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_PARSE_CONCURRENCY));
    let parse_fn = std::sync::Arc::new(parse_fn);
    let mut handles = Vec::with_capacity(files.len());

    for (path, data) in files {
        let sem = semaphore.clone();
        let parser = parse_fn.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem
                .acquire()
                .await
                .expect("Semaphore closed unexpectedly");
            let text = tokio::task::spawn_blocking(move || parser(&data))
                .await
                .unwrap_or_else(|e| Err(format!("任务执行失败: {}", e)));
            ParsedFileResult { path, text }
        });
        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(ParsedFileResult {
                path: String::new(),
                text: Err(format!("任务 join 失败: {}", e)),
            }),
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Two empty strings → stats all zero, no hunks
    #[test]
    fn test_diff_empty_strings() {
        let result = compute_diff("", "");
        assert_eq!(result.stats.additions, 0);
        assert_eq!(result.stats.deletions, 0);
        assert_eq!(result.stats.unchanged, 0);
        assert!(result.hunks.is_empty());
    }

    // Test 2: Same text → additions=0, deletions=0, unchanged>0
    #[test]
    fn test_diff_same_text() {
        let text = "line 1\nline 2\nline 3\n";
        let result = compute_diff(text, text);
        assert_eq!(result.stats.additions, 0);
        assert_eq!(result.stats.deletions, 0);
        assert!(result.stats.unchanged > 0);
        assert_eq!(result.stats.unchanged, 3);
        assert!(result.hunks.is_empty(), "无变化时不应输出 hunk");
    }

    // Test 3: Pure insertion (old empty) → deletions=0
    #[test]
    fn test_diff_pure_insertion() {
        let result = compute_diff("", "line 1\nline 2\n");
        assert_eq!(result.stats.deletions, 0);
        assert_eq!(result.stats.additions, 2);
        assert_eq!(result.stats.unchanged, 0);
    }

    // Test 4: Pure deletion (new empty) → additions=0
    #[test]
    fn test_diff_pure_deletion() {
        let result = compute_diff("line 1\nline 2\n", "");
        assert_eq!(result.stats.additions, 0);
        assert_eq!(result.stats.deletions, 2);
        assert_eq!(result.stats.unchanged, 0);
    }

    // Test 5: Mixed change stats count correctly
    #[test]
    fn test_diff_mixed_changes() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified 2\nline 3\nnew line 4\n";
        let result = compute_diff(old, new);
        // "line 1" and "line 3" are unchanged, "line 2" deleted, "modified 2" and "new line 4" added
        assert_eq!(result.stats.unchanged, 2);
        assert_eq!(result.stats.deletions, 1);
        assert_eq!(result.stats.additions, 2);
    }

    // Test 6: Unchanged regions away from edits are omitted like git unified diff
    #[test]
    fn test_diff_uses_unified_context() {
        let old = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let new = old.replace("line 10\n", "line ten\n");
        let result = compute_diff(&old, &new);

        assert_eq!(result.hunks.len(), 1);
        let hunk = &result.hunks[0];
        assert_eq!(hunk.old_start, 7);
        assert_eq!(hunk.new_start, 7);
        assert_eq!(hunk.changes.len(), 8);
        assert_eq!(hunk.changes.first().unwrap().content, "line 7");
        assert_eq!(hunk.changes.last().unwrap().content, "line 13");
    }

    // Test 7: Single-line change has correct line numbers
    #[test]
    fn test_diff_single_line_numbers() {
        let old = "aaa\n";
        let new = "bbb\n";
        let result = compute_diff(old, new);
        assert!(!result.hunks.is_empty());
        let first_change = &result.hunks[0].changes[0];
        // First change should be a delete of old line 1
        assert_eq!(first_change.change_type, "delete");
        assert_eq!(first_change.old_line, Some(1));
        assert_eq!(first_change.new_line, None);
    }

    // Test 8: Multi-line change has correct change_type markers
    #[test]
    fn test_diff_change_types() {
        let old = "keep\nremove\n";
        let new = "keep\nadd\n";
        let result = compute_diff(old, new);
        let all_changes: Vec<&DiffChange> =
            result.hunks.iter().flat_map(|h| h.changes.iter()).collect();

        let equal_changes: Vec<_> = all_changes
            .iter()
            .filter(|c| c.change_type == "equal")
            .collect();
        let delete_changes: Vec<_> = all_changes
            .iter()
            .filter(|c| c.change_type == "delete")
            .collect();
        let add_changes: Vec<_> = all_changes
            .iter()
            .filter(|c| c.change_type == "add")
            .collect();

        assert_eq!(equal_changes.len(), 1, "Expected 1 equal change");
        assert_eq!(delete_changes.len(), 1, "Expected 1 delete change");
        assert_eq!(add_changes.len(), 1, "Expected 1 add change");

        // Verify equal has both line numbers, delete has only old, add has only new
        assert!(equal_changes[0].old_line.is_some());
        assert!(equal_changes[0].new_line.is_some());
        assert!(delete_changes[0].old_line.is_some());
        assert!(delete_changes[0].new_line.is_none());
        assert!(add_changes[0].old_line.is_none());
        assert!(add_changes[0].new_line.is_some());
    }

    #[test]
    fn test_diff_separates_distant_changes_into_multiple_hunks() {
        let old = (1..=30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let new = old
            .replace("line 5\n", "line five\n")
            .replace("line 25\n", "line twenty five\n");

        let result = compute_diff(&old, &new);

        assert_eq!(result.hunks.len(), 2);
        assert_eq!(result.hunks[0].old_start, 2);
        assert_eq!(result.hunks[1].old_start, 22);
    }

    #[test]
    fn test_diff_pure_insert_hunk_uses_zero_old_start() {
        let result = compute_diff("", "line 1\nline 2\n");

        assert_eq!(result.hunks.len(), 1);
        assert_eq!(result.hunks[0].old_start, 0);
        assert_eq!(result.hunks[0].old_lines, 0);
        assert_eq!(result.hunks[0].new_start, 1);
        assert_eq!(result.hunks[0].new_lines, 2);
    }
}
