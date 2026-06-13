//! 增强差异引擎模块 — 二进制差异比较

use crate::diff::types::BinaryDiffResult;
use crate::error::AppError;
use sha2::{Digest, Sha256};

#[allow(dead_code)]
pub struct BinaryDiffEngine;

#[allow(dead_code)]
impl BinaryDiffEngine {
    /// 比较两个二进制内容
    pub fn compare(
        old: &[u8],
        new: &[u8],
    ) -> Result<BinaryDiffResult, AppError> {
        let old_hash = Self::compute_hash(old);
        let new_hash = Self::compute_hash(new);

        let identical = old_hash == new_hash;
        let size_change = new.len() as i64 - old.len() as i64;
        let similarity = Self::calculate_similarity(old, new);

        let summary = if identical {
            "文件内容完全相同".to_string()
        } else {
            let change_str = if size_change > 0 {
                format!("+{} bytes", size_change)
            } else {
                format!("{} bytes", size_change)
            };
            format!(
                "文件已修改，大小变化: {} ({} → {} bytes)",
                change_str,
                old.len(),
                new.len()
            )
        };

        Ok(BinaryDiffResult {
            identical,
            size_change,
            old_size: old.len() as u64,
            new_size: new.len() as u64,
            old_hash,
            new_hash,
            similarity,
            summary,
        })
    }

    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_similarity(old: &[u8], new: &[u8]) -> f64 {
        if old.is_empty() && new.is_empty() {
            return 1.0;
        }

        let max_size = old.len().max(new.len()) as f64;
        let min_size = old.len().min(new.len()) as f64;

        // Size ratio gives an upper bound; also check actual byte matches
        // in the overlapping region for content-aware similarity.
        let overlap = min_size as usize;
        if overlap == 0 {
            return 0.0;
        }

        let matching = old[..overlap]
            .iter()
            .zip(new[..overlap].iter())
            .filter(|(a, b)| a == b)
            .count() as f64;

        // Blend: matching bytes in overlap + size penalty for size difference
        matching / max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_content() {
        let data = b"hello world";
        let result = BinaryDiffEngine::compare(data, data).unwrap();
        assert!(result.identical);
        assert_eq!(result.size_change, 0);
        assert_eq!(result.similarity, 1.0);
        assert_eq!(result.old_hash, result.new_hash);
    }

    #[test]
    fn test_different_content() {
        let old = b"hello";
        let new = b"world!";
        let result = BinaryDiffEngine::compare(old, new).unwrap();
        assert!(!result.identical);
        assert_eq!(result.size_change, 1);
        assert_ne!(result.old_hash, result.new_hash);
    }

    #[test]
    fn test_size_change_positive() {
        let result = BinaryDiffEngine::compare(b"abc", b"abcdef").unwrap();
        assert_eq!(result.size_change, 3);
        assert_eq!(result.old_size, 3);
        assert_eq!(result.new_size, 6);
    }

    #[test]
    fn test_size_change_negative() {
        let result = BinaryDiffEngine::compare(b"abcdef", b"abc").unwrap();
        assert_eq!(result.size_change, -3);
    }

    #[test]
    fn test_empty_content() {
        let result = BinaryDiffEngine::compare(b"", b"").unwrap();
        assert!(result.identical);
        assert_eq!(result.similarity, 1.0);
    }

    #[test]
    fn test_one_empty() {
        let result = BinaryDiffEngine::compare(b"abc", b"").unwrap();
        assert!(!result.identical);
        assert_eq!(result.similarity, 0.0);
    }

    #[test]
    fn test_compare_different_summary() {
        let old = b"hello";
        let new = b"hello world";
        let result = BinaryDiffEngine::compare(old, new).unwrap();

        assert!(!result.identical);
        assert_eq!(result.size_change, 6); // " world" = 6 bytes
        assert_eq!(result.old_size, 5);
        assert_eq!(result.new_size, 11);
        assert_ne!(result.old_hash, result.new_hash);
        assert!(result.similarity < 1.0);
        assert!(result.summary.contains("+6 bytes"));
    }

    #[test]
    fn test_compare_smaller_new_summary() {
        let old = b"hello world";
        let new = b"hello";
        let result = BinaryDiffEngine::compare(old, new).unwrap();

        assert!(!result.identical);
        assert_eq!(result.size_change, -6);
        assert!(result.summary.contains("-6 bytes"));
    }

    #[test]
    fn test_compute_hash_consistent() {
        let data = b"test data";
        let hash1 = BinaryDiffEngine::compute_hash(data);
        let hash2 = BinaryDiffEngine::compute_hash(data);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_calculate_similarity_same_size() {
        let data = b"hello";
        let similarity = BinaryDiffEngine::calculate_similarity(data, data);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_calculate_similarity_different_size() {
        let old = b"hi";
        let new = b"hello";
        let similarity = BinaryDiffEngine::calculate_similarity(old, new);
        // Content-aware: 'h' matches, 'i'≠'e' → 1/5 = 0.2
        assert_eq!(similarity, 0.2);
    }
}
