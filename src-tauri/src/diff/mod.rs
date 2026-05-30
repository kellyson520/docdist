use similar::{ChangeTag, TextDiff};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffChange {
    pub change_type: String,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
    pub unchanged: u32,
}

pub fn compute_diff(
    old_text: &str,
    new_text: &str,
) -> DiffResult {
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
            ChangeTag::Equal | ChangeTag::Delete => {
                old_line += 1
            }
            _ => {}
        }
        match change.tag() {
            ChangeTag::Equal | ChangeTag::Insert => {
                new_line += 1
            }
            _ => {}
        }

        current_hunk.old_lines += 1;
        current_hunk.new_lines += 1;

        // Split into hunks of ~20 lines
        if current_hunk.changes.len() >= 20
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
