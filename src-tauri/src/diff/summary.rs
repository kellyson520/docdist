//! 差异摘要生成模块

use super::types::*;
use super::{DiffHunk, DiffResult};

pub struct SummaryGenerator {
    max_changes: usize,
    _max_snippet_lines: usize,
}

impl SummaryGenerator {
    pub fn new() -> Self {
        Self {
            max_changes: 100,
            _max_snippet_lines: 10,
        }
    }

    /// 从差异结果生成摘要
    pub fn generate(&self, diff_result: &DiffResult) -> DiffSummary {
        let changes = self.extract_changes(diff_result);
        let distribution = self.calculate_distribution(&changes);
        let affected_regions = self.detect_affected_regions(&changes);
        let ai_summary = self.generate_ai_summary(&changes);

        let modifications = 0usize;
        let additions = diff_result.stats.additions as usize;
        let deletions = diff_result.stats.deletions as usize;
        let unchanged = diff_result.stats.unchanged as usize;
        let total = additions + deletions + unchanged;

        DiffSummary {
            stats: DiffStats {
                additions,
                deletions,
                modifications,
                unchanged,
                total,
            },
            changes,
            change_distribution: distribution,
            affected_regions,
            ai_summary,
        }
    }

    fn extract_changes(&self, diff_result: &DiffResult) -> Vec<ChangeSummary> {
        let mut changes = Vec::new();
        let mut change_id = 0u32;

        for hunk in &diff_result.hunks {
            self.extract_hunk_changes(hunk, &mut changes, &mut change_id);
            if changes.len() >= self.max_changes {
                break;
            }
        }

        changes
    }

    fn extract_hunk_changes(
        &self,
        hunk: &DiffHunk,
        changes: &mut Vec<ChangeSummary>,
        change_id: &mut u32,
    ) {
        for change in &hunk.changes {
            if changes.len() >= self.max_changes {
                break;
            }

            let change_type = match change.change_type.as_str() {
                "add" => ChangeType::Addition,
                "delete" => ChangeType::Deletion,
                _ => continue,
            };

            let line = if change_type == ChangeType::Addition {
                change.new_line.unwrap_or(0)
            } else {
                change.old_line.unwrap_or(0)
            };

            let label = if change_type == ChangeType::Addition {
                "新增"
            } else {
                "删除"
            };

            let snippet_text: String =
                change.content.chars().take(50).collect();

            changes.push(ChangeSummary {
                id: *change_id,
                change_type,
                location: ChangeLocation {
                    file_path: String::new(),
                    start_line: line,
                    end_line: line,
                    start_col: None,
                    end_col: None,
                    region_description: None,
                },
                description: format!("{}: {}", label, snippet_text),
                snippet: Some(change.content.clone()),
                line_count: 1,
            });

            *change_id += 1;
        }
    }

    fn calculate_distribution(
        &self,
        changes: &[ChangeSummary],
    ) -> ChangeDistribution {
        let mut dist = ChangeDistribution {
            additions: 0,
            deletions: 0,
            modifications: 0,
            moves: 0,
            renames: 0,
        };

        for change in changes {
            match change.change_type {
                ChangeType::Addition => dist.additions += 1,
                ChangeType::Deletion => dist.deletions += 1,
                ChangeType::Modification => dist.modifications += 1,
                ChangeType::Move => dist.moves += 1,
                ChangeType::Rename => dist.renames += 1,
                _ => {}
            }
        }

        dist
    }

    fn detect_affected_regions(
        &self,
        changes: &[ChangeSummary],
    ) -> Vec<AffectedRegion> {
        let mut regions = Vec::new();
        let mut current_region: Option<AffectedRegion> = None;

        for change in changes {
            if let Some(ref mut region) = current_region {
                if change.location.start_line <= region.end_line + 5 {
                    region.end_line = change.location.end_line;
                    region.change_lines += change.line_count;
                    continue;
                }
            }

            if let Some(r) = current_region.take() {
                regions.push(r);
            }

            current_region = Some(AffectedRegion {
                name: format!("区域 #{}", regions.len() + 1),
                region_type: RegionType::Paragraph,
                change_type: change.change_type.clone(),
                change_lines: change.line_count,
                start_line: change.location.start_line,
                end_line: change.location.end_line,
            });
        }

        if let Some(r) = current_region {
            regions.push(r);
        }

        regions
    }

    fn generate_ai_summary(&self, changes: &[ChangeSummary]) -> Option<String> {
        if changes.is_empty() {
            return Some("文件无变化".to_string());
        }

        let additions = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Addition)
            .count();
        let deletions = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Deletion)
            .count();

        Some(format!(
            "共 {} 处变更：新增 {} 处，删除 {} 处",
            changes.len(),
            additions,
            deletions
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::DiffChange;
    use crate::diff::{DiffResult, DiffStats as DiffResultStats};

    fn make_diff_result() -> DiffResult {
        DiffResult {
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 3,
                changes: vec![
                    DiffChange {
                        change_type: "delete".to_string(),
                        content: "old line".to_string(),
                        old_line: Some(1),
                        new_line: None,
                    },
                    DiffChange {
                        change_type: "add".to_string(),
                        content: "new line".to_string(),
                        old_line: None,
                        new_line: Some(1),
                    },
                ],
            }],
            stats: DiffResultStats {
                additions: 1,
                deletions: 1,
                unchanged: 0,
            },
        }
    }

    #[test]
    fn test_generate_summary() {
        let gen = SummaryGenerator::new();
        let result = gen.generate(&make_diff_result());
        assert_eq!(result.stats.additions, 1);
        assert_eq!(result.stats.deletions, 1);
        assert_eq!(result.changes.len(), 2);
        assert!(result.ai_summary.is_some());
    }

    #[test]
    fn test_empty_diff() {
        let gen = SummaryGenerator::new();
        let empty = DiffResult {
            hunks: vec![],
            stats: DiffResultStats {
                additions: 0,
                deletions: 0,
                unchanged: 0,
            },
        };
        let result = gen.generate(&empty);
        assert_eq!(result.changes.len(), 0);
        assert_eq!(result.ai_summary.unwrap(), "文件无变化");
    }

    #[test]
    fn test_distribution() {
        let gen = SummaryGenerator::new();
        let result = gen.generate(&make_diff_result());
        assert_eq!(result.change_distribution.additions, 1);
        assert_eq!(result.change_distribution.deletions, 1);
    }
}
