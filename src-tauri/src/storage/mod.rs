use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

pub fn compute_hash(data: &[u8]) -> String {
    format!("{:016x}", xxh3_64(data))
}

/// 辅助：从 hash 取前两个字符作为子目录名，hash 长度不足时返回错误
fn hash_prefix(hash: &str) -> Result<&str, crate::error::AppError> {
    if hash.len() < 2 {
        return Err(crate::error::AppError::Other(format!(
            "chunk hash too short for directory sharding: '{}'",
            hash
        )));
    }
    Ok(&hash[..2])
}

/// 存储文件到分块存储，返回 (chunk_hashes, chunk_sizes, file_size, file_checksum)
pub fn store_file(
    base_dir: &Path,
    file_path: &Path,
    chunk_size: usize,
) -> Result<(Vec<String>, Vec<usize>, u64, String), crate::error::AppError> {
    let content = std::fs::read(file_path)?;
    let file_size = content.len() as u64;
    let file_checksum = compute_hash(&content);
    let mut chunk_hashes = Vec::new();
    let mut chunk_sizes = Vec::new();

    for chunk in content.chunks(chunk_size) {
        let hash = compute_hash(chunk);
        let prefix = hash_prefix(&hash)?;
        let chunk_dir = base_dir.join(prefix);
        let chunk_path = chunk_dir.join(&hash);

        if !chunk_path.exists() {
            std::fs::create_dir_all(&chunk_dir)?;
            std::fs::write(&chunk_path, chunk)?;
        }

        chunk_hashes.push(hash);
        chunk_sizes.push(chunk.len());
    }

    Ok((chunk_hashes, chunk_sizes, file_size, file_checksum))
}

/// 从分块存储恢复文件
pub fn restore_file(
    base_dir: &Path,
    chunk_hashes: &[String],
    output_path: &Path,
) -> Result<(), crate::error::AppError> {
    let mut content = Vec::new();

    for hash in chunk_hashes {
        let prefix = hash_prefix(hash)?;
        let chunk_path = base_dir.join(prefix).join(hash);
        if !chunk_path.exists() {
            return Err(crate::error::AppError::Other(format!(
                "分块不存在: {}",
                hash
            )));
        }
        let chunk_data = std::fs::read(&chunk_path)?;
        content.extend_from_slice(&chunk_data);
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(output_path, &content)?;
    Ok(())
}

/// 清理不再被任何存档引用的孤儿 chunks
/// active_hashes 是当前数据库中所有被引用的 chunk hash 集合
pub fn cleanup_orphan_chunks(
    base_dir: &Path,
    active_hashes: &[String],
) -> Result<CleanupStats, crate::error::AppError> {
    let active_set: std::collections::HashSet<String> =
        active_hashes.iter().cloned().collect();
    let mut stats = CleanupStats::default();

    if !base_dir.exists() {
        return Ok(stats);
    }

    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let dir_path = entry.path();

        for chunk_entry in std::fs::read_dir(&dir_path)? {
            let chunk_entry = chunk_entry?;
            let name = chunk_entry.file_name().to_string_lossy().to_string();

            if !chunk_entry.file_type()?.is_file() {
                continue;
            }

            if !active_set.contains(&name) {
                let size = chunk_entry.metadata()?.len();
                std::fs::remove_file(chunk_entry.path())?;
                stats.removed_count += 1;
                stats.removed_bytes += size;
                tracing::debug!("Removed orphan chunk: {}", name);
            } else {
                stats.kept_count += 1;
            }
        }

        // 如果目录为空，删除它
        if dir_path.read_dir()?.next().is_none() {
            std::fs::remove_dir(&dir_path)?;
        }
    }

    Ok(stats)
}

/// 计算存储空间使用统计
pub fn get_storage_usage(
    base_dir: &Path,
) -> Result<StorageUsage, crate::error::AppError> {
    let mut usage = StorageUsage::default();

    if !base_dir.exists() {
        return Ok(usage);
    }

    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        for chunk_entry in std::fs::read_dir(entry.path())? {
            let chunk_entry = chunk_entry?;
            if chunk_entry.file_type()?.is_file() {
                let size = chunk_entry.metadata()?.len();
                usage.total_chunks += 1;
                usage.total_bytes += size;
            }
        }
    }

    Ok(usage)
}

/// 验证 chunk 完整性
pub fn verify_chunk(
    base_dir: &Path,
    hash: &str,
) -> Result<bool, crate::error::AppError> {
    let prefix = hash_prefix(hash)?;
    let chunk_path = base_dir.join(prefix).join(hash);
    if !chunk_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read(&chunk_path)?;
    let actual_hash = compute_hash(&content);
    Ok(actual_hash == hash)
}

#[derive(Debug, Default, serde::Serialize)]
pub struct CleanupStats {
    pub removed_count: usize,
    pub removed_bytes: u64,
    pub kept_count: usize,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct StorageUsage {
    pub total_chunks: u64,
    pub total_bytes: u64,
}
