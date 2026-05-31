pub mod engine;
pub mod parsers; // 预留，T2 会创建 parsers/mod.rs
pub mod summary; // 预留，T4 会创建 summary.rs
pub mod types; // 预留，T4 会创建 engine.rs

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

/// Maximum number of changes per hunk before splitting.
const HUNK_MAX_LINES: usize = 20;

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

    let mut hunks = Vec::new();
    let mut current_hunk = DiffHunk {
        old_start: 1,
        old_lines: 0,
        new_start: 1,
        new_lines: 0,
        changes: Vec::new(),
    };

    let mut additions: u32 = 0;
    let mut deletions: u32 = 0;
    let mut unchanged: u32 = 0;
    let mut old_line: u32 = 1;
    let mut new_line: u32 = 1;

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

        current_hunk.changes.push(DiffChange {
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

        match change.tag() {
            ChangeTag::Equal => {
                current_hunk.old_lines += 1;
                current_hunk.new_lines += 1;
            }
            ChangeTag::Delete => {
                current_hunk.old_lines += 1;
            }
            ChangeTag::Insert => {
                current_hunk.new_lines += 1;
            }
        }

        if current_hunk.changes.len() >= HUNK_MAX_LINES
            && change.tag() == ChangeTag::Equal
        {
            hunks.push(current_hunk.clone());
            current_hunk = DiffHunk {
                old_start: old_line,
                old_lines: 0,
                new_start: new_line,
                new_lines: 0,
                changes: Vec::new(),
            };
        }
    }

    if !current_hunk.changes.is_empty() {
        hunks.push(current_hunk);
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

    // Test 6: More than 20 lines → hunk splits correctly
    #[test]
    fn test_diff_hunk_split_at_20_lines() {
        // Build two texts where all lines are equal (30 lines) to trigger hunk split
        let lines: Vec<String> =
            (1..=30).map(|i| format!("line {}", i)).collect();
        let text = lines.join("\n") + "\n";
        let result = compute_diff(&text, &text);
        // All lines are equal, so after 20 lines the hunk should split
        assert!(
            result.hunks.len() >= 2,
            "Expected at least 2 hunks for 30 equal lines, got {}",
            result.hunks.len()
        );
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
}
