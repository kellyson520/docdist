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
/// 使用流式分块读取，避免一次性将整个文件加载到内存中，防止大文件 OOM。
pub fn store_file(
    base_dir: &Path,
    file_path: &Path,
    chunk_size: usize,
) -> Result<(Vec<String>, Vec<usize>, u64, String), crate::error::AppError> {
    use std::io::Read;

    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = vec![0u8; chunk_size];
    let mut chunk_hashes = Vec::new();
    let mut chunk_sizes = Vec::new();
    let mut file_hasher = blake3::Hasher::new();
    let mut file_size: u64 = 0;
    let mut filled: usize = 0; // buffer 中当前有效数据量

    loop {
        // 从文件读取数据填充 buffer 的空余部分
        loop {
            let n = file.read(&mut buffer[filled..])?;
            if n == 0 {
                break;
            }
            filled += n;
        }

        // 没有数据了（EOF 且无剩余），退出外层循环
        if filled == 0 {
            break;
        }

        // 处理 buffer 中完整的 chunk
        let mut consumed = 0;
        while consumed + chunk_size <= filled {
            let chunk = &buffer[consumed..consumed + chunk_size];
            file_hasher.update(chunk);
            file_size += chunk_size as u64;

            let hash = compute_hash(chunk);
            let prefix = hash_prefix(&hash)?;
            let chunk_dir = base_dir.join(prefix);
            let chunk_path = chunk_dir.join(&hash);

            if !chunk_path.exists() {
                std::fs::create_dir_all(&chunk_dir)?;
                std::fs::write(&chunk_path, chunk)?;
            }

            chunk_hashes.push(hash);
            chunk_sizes.push(chunk_size);
            consumed += chunk_size;
        }

        // 处理剩余的最后一个 chunk（可能小于 chunk_size）
        if consumed < filled {
            let chunk = &buffer[consumed..filled];
            file_hasher.update(chunk);
            file_size += chunk.len() as u64;

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

        // 重置 buffer，准备下一轮读取
        filled = 0;
    }

    let file_checksum = file_hasher.finalize().to_hex().to_string();
    Ok((chunk_hashes, chunk_sizes, file_size, file_checksum))
}

/// 从分块存储恢复文件
/// 使用流式写入，避免一次性将整个文件加载到内存中。
pub fn restore_file(
    base_dir: &Path,
    chunk_hashes: &[String],
    output_path: &Path,
) -> Result<(), crate::error::AppError> {
    use std::io::Write;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to a temp file first, then atomically rename.
    // This prevents data loss if restore fails midway (e.g. missing chunk,
    // disk full) — the original file at output_path is not destroyed.
    let temp_path = output_path.with_extension({
        let mut ext = output_path
            .extension()
            .map(|e| e.to_os_string())
            .unwrap_or_default();
        ext.push(".tmprestore");
        ext
    });

    // Use a helper closure so we can clean up the temp file on any error
    let result: Result<(), crate::error::AppError> = (|| {
        let out_file = std::fs::File::create(&temp_path)?;
        let mut out_file = std::io::BufWriter::new(out_file);

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
            out_file.write_all(&chunk_data)?;
        }

        out_file.flush()?;
        Ok(())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
        return result;
    }

    // Atomic rename: either the full file exists or the old one is preserved
    std::fs::rename(&temp_path, output_path)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ========== compute_hash 测试 ==========

    #[test]
    fn test_compute_hash_deterministic() {
        let data = b"hello world";
        let hash1 = compute_hash(data);
        let hash2 = compute_hash(data);
        assert_eq!(hash1, hash2, "相同输入应产生相同 hash");
    }

    #[test]
    fn test_compute_hash_output_length() {
        let data = b"any data for testing";
        let hash = compute_hash(data);
        assert_eq!(
            hash.len(),
            16,
            "hash 输出应为 16 个 hex 字符，实际长度: {}",
            hash.len()
        );
        // 额外验证全部为 hex 字符
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash 应仅包含 hex 字符: {}",
            hash
        );
    }

    // ========== store_file 测试 ==========

    #[test]
    fn test_store_file_small_single_chunk() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("small.txt");

        // 写入一个小文件（< chunk_size）
        std::fs::write(&file_path, b"small content").unwrap();

        let (chunk_hashes, chunk_sizes, file_size, checksum) =
            store_file(&base_dir, &file_path, 4096).unwrap();

        assert_eq!(chunk_hashes.len(), 1, "小文件应只有 1 个 chunk");
        assert_eq!(chunk_sizes.len(), 1);
        assert_eq!(
            file_size,
            b"small content".len() as u64,
            "文件大小应与内容一致"
        );
        assert!(!checksum.is_empty());
        // 验证 chunk 文件实际存在
        let prefix = hash_prefix(&chunk_hashes[0]).unwrap();
        let chunk_path = base_dir.join(prefix).join(&chunk_hashes[0]);
        assert!(chunk_path.exists(), "chunk 文件应已写入磁盘");
    }

    #[test]
    fn test_store_file_large_multiple_chunks() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("large.bin");

        // 写入一个 10000 字节的文件，chunk_size=4096 → ceil(10000/4096) = 3
        let content: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        std::fs::write(&file_path, &content).unwrap();

        let (chunk_hashes, chunk_sizes, file_size, _checksum) =
            store_file(&base_dir, &file_path, 4096).unwrap();

        let expected_chunks = (10000 + 4096 - 1) / 4096; // ceil(10000/4096) = 3
        assert_eq!(
            chunk_hashes.len(),
            expected_chunks,
            "10000 字节文件分 4096 chunk 应有 {} 个",
            expected_chunks
        );
        assert_eq!(chunk_sizes.len(), expected_chunks);
        assert_eq!(file_size, 10000);

        // 验证 chunk 大小之和等于文件大小
        let total_chunk_size: usize = chunk_sizes.iter().sum();
        assert_eq!(
            total_chunk_size, 10000,
            "所有 chunk 大小之和应等于文件大小"
        );

        // 验证最后一个 chunk 大小 < chunk_size
        assert!(
            *chunk_sizes.last().unwrap() <= 4096,
            "最后一个 chunk 不应超过 chunk_size"
        );
    }

    #[test]
    fn test_store_file_dedup() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        // 两个文件内容完全相同
        let content = b"identical content for dedup test";
        std::fs::write(&file1, content).unwrap();
        std::fs::write(&file2, content).unwrap();

        let (hashes1, _, _, _) = store_file(&base_dir, &file1, 4096).unwrap();
        let (hashes2, _, _, _) = store_file(&base_dir, &file2, 4096).unwrap();

        assert_eq!(hashes1, hashes2, "相同内容应产生相同 chunk hash");

        // 验证磁盘上只有一份 chunk 文件
        let prefix = hash_prefix(&hashes1[0]).unwrap();
        let chunk_path = base_dir.join(prefix).join(&hashes1[0]);
        assert!(chunk_path.exists());

        // 统计 chunks 子目录中的文件数量，应只有 1 个
        let chunk_dir = base_dir.join(prefix);
        let file_count: usize = std::fs::read_dir(&chunk_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .count();
        assert_eq!(file_count, 1, "去重后磁盘上应只有 1 个 chunk 文件");
    }

    // ========== restore_file 测试 ==========

    #[test]
    fn test_restore_file_correct_content() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("original.txt");
        let restore_path = dir.path().join("restored.txt");

        let content = b"Hello, DocDist! This is a test file for restore.";
        std::fs::write(&file_path, content).unwrap();

        let (chunk_hashes, _, _, _) =
            store_file(&base_dir, &file_path, 4096).unwrap();

        restore_file(&base_dir, &chunk_hashes, &restore_path).unwrap();

        let restored = std::fs::read(&restore_path).unwrap();
        assert_eq!(
            content.to_vec(),
            restored,
            "恢复的文件内容应与原始文件完全一致"
        );
    }

    #[test]
    fn test_restore_file_missing_chunk_error() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let restore_path = dir.path().join("restored.txt");

        // 使用一个不存在的 hash
        let fake_hashes = vec!["0000000000000000".to_string()];

        let result = restore_file(&base_dir, &fake_hashes, &restore_path);
        assert!(result.is_restore_err(), "缺失 chunk 应返回错误");
    }

    // ========== cleanup_orphan_chunks 测试 ==========

    #[test]
    fn test_cleanup_orphan_chunks_removes_inactive() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");

        // 手动创建一些 chunk 文件
        let prefix = "ab";
        let active_hash = format!("ab{}", "1".repeat(14)); // "ab11111111111111"
        let orphan_hash = format!("ab{}", "2".repeat(14)); // "ab22222222222222"

        let chunk_dir = base_dir.join(prefix);
        std::fs::create_dir_all(&chunk_dir).unwrap();
        std::fs::write(chunk_dir.join(&active_hash), b"active data").unwrap();
        std::fs::write(chunk_dir.join(&orphan_hash), b"orphan data").unwrap();

        let active_hashes = vec![active_hash];
        let stats = cleanup_orphan_chunks(&base_dir, &active_hashes).unwrap();

        assert_eq!(stats.removed_count, 1, "应删除 1 个孤儿 chunk");
        assert_eq!(stats.kept_count, 1, "应保留 1 个活跃 chunk");
        assert!(stats.removed_bytes > 0, "删除的字节数应大于 0");
    }

    #[test]
    fn test_cleanup_orphan_chunks_keeps_active() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");

        let prefix = "cd";
        let hash1 = format!("cd{}", "a".repeat(14));
        let hash2 = format!("cd{}", "b".repeat(14));

        let chunk_dir = base_dir.join(prefix);
        std::fs::create_dir_all(&chunk_dir).unwrap();
        std::fs::write(chunk_dir.join(&hash1), b"data1").unwrap();
        std::fs::write(chunk_dir.join(&hash2), b"data2").unwrap();

        // 两个都是活跃的
        let active_hashes = vec![hash1.clone(), hash2.clone()];
        let stats = cleanup_orphan_chunks(&base_dir, &active_hashes).unwrap();

        assert_eq!(stats.removed_count, 0, "不应删除任何 chunk");
        assert_eq!(stats.kept_count, 2, "应保留 2 个 chunk");

        // 验证文件仍然存在
        assert!(chunk_dir.join(&hash1).exists());
        assert!(chunk_dir.join(&hash2).exists());
    }

    #[test]
    fn test_cleanup_orphan_chunks_removes_empty_dir() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");

        let prefix = "ef";
        let orphan_hash = format!("ef{}", "c".repeat(14));

        let chunk_dir = base_dir.join(prefix);
        std::fs::create_dir_all(&chunk_dir).unwrap();
        std::fs::write(chunk_dir.join(&orphan_hash), b"orphan").unwrap();

        // 没有活跃 hash，所有 chunk 都将被删除
        let active_hashes: Vec<String> = vec![];
        let stats = cleanup_orphan_chunks(&base_dir, &active_hashes).unwrap();

        assert_eq!(stats.removed_count, 1);
        // 目录应被自动删除
        assert!(!chunk_dir.exists(), "空目录应被自动删除");
    }

    // ========== verify_chunk 测试 ==========

    #[test]
    fn test_verify_chunk_correct_hash() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("test.txt");

        let content = b"verify me!";
        std::fs::write(&file_path, content).unwrap();

        let (chunk_hashes, _, _, _) =
            store_file(&base_dir, &file_path, 4096).unwrap();
        let hash = &chunk_hashes[0];

        let result = verify_chunk(&base_dir, hash).unwrap();
        assert!(result, "正确的 hash 应返回 true");
    }

    #[test]
    fn test_verify_chunk_tampered_returns_false() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");

        // 手动创建一个 chunk 文件，内容与 hash 不匹配
        let prefix = "ab";
        let hash = compute_hash(b"original content");
        // 实际写入的是不同内容
        let chunk_dir = base_dir.join(prefix);
        std::fs::create_dir_all(&chunk_dir).unwrap();
        std::fs::write(chunk_dir.join(&hash), b"TAMPERED content").unwrap();

        let result = verify_chunk(&base_dir, &hash).unwrap();
        assert!(!result, "被篡改的 chunk 应返回 false");
    }

    #[test]
    fn test_verify_chunk_nonexistent_returns_false() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");

        let result = verify_chunk(&base_dir, "0000000000000000").unwrap();
        assert!(!result, "不存在的 chunk 应返回 false");
    }

    // ========== get_storage_usage 测试 ==========

    #[test]
    fn test_get_storage_usage_correct() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("usage_test.txt");

        // 存储两个不同内容的文件到不同 chunk
        let content1 = b"first file content for usage";
        let content2 = b"second file content for usage test";
        std::fs::write(&file_path, content1).unwrap();
        let (_hashes1, _, _, _) =
            store_file(&base_dir, &file_path, 4096).unwrap();
        std::fs::write(&file_path, content2).unwrap();
        let (_hashes2, _, _, _) =
            store_file(&base_dir, &file_path, 4096).unwrap();

        let usage = get_storage_usage(&base_dir).unwrap();

        // 两个不同内容应产生 2 个 chunk
        assert_eq!(usage.total_chunks, 2, "应有 2 个 chunk");
        assert_eq!(
            usage.total_bytes,
            (content1.len() + content2.len()) as u64,
            "总字节数应为两个文件内容大小之和"
        );
    }

    #[test]
    fn test_get_storage_usage_empty_dir() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("empty_chunks");

        let usage = get_storage_usage(&base_dir).unwrap();
        assert_eq!(usage.total_chunks, 0);
        assert_eq!(usage.total_bytes, 0);
    }

    // ========== hash_prefix 测试 ==========

    #[test]
    fn test_hash_prefix_too_short_error() {
        // 空字符串
        let result = hash_prefix("");
        assert!(result.is_err(), "空 hash 应返回错误");

        // 单字符
        let result = hash_prefix("a");
        assert!(result.is_err(), "单字符 hash 应返回错误");

        // 验证错误信息包含中文提示
        let err = hash_prefix("").unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("chunk hash too short"),
            "错误信息应包含 'chunk hash too short'"
        );
    }

    #[test]
    fn test_hash_prefix_valid() {
        let result = hash_prefix("abcdef1234567890");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ab");

        // 正好 2 字符
        let result = hash_prefix("xy");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "xy");
    }

    // ========== 多 chunk restore 集成测试 ==========

    #[test]
    fn test_store_and_restore_large_file_roundtrip() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().join("chunks");
        let file_path = dir.path().join("large_original.bin");
        let restore_path = dir.path().join("large_restored.bin");

        // 创建 20000 字节的内容，chunk_size=4096 → 5 个 chunk
        let content: Vec<u8> = (0..20000).map(|i| (i % 251) as u8).collect(); // 251 为质数
        std::fs::write(&file_path, &content).unwrap();

        let (chunk_hashes, _chunk_sizes, file_size, checksum) =
            store_file(&base_dir, &file_path, 4096).unwrap();

        assert_eq!(chunk_hashes.len(), 5);
        assert_eq!(file_size, 20000);

        // 恢复文件
        restore_file(&base_dir, &chunk_hashes, &restore_path).unwrap();

        let restored = std::fs::read(&restore_path).unwrap();
        assert_eq!(content, restored.as_slice(), "大文件 roundtrip 内容应一致");

        // 验证 checksum (file_checksum 现在使用 blake3 增量计算)
        assert_eq!(blake3::hash(&restored).to_hex().to_string(), checksum);
    }

    // ========== 辅助 trait 测试 ==========
    // restore_file 错误类型是 AppError::Other，使用 is_err 即可

    trait ResultExt<T> {
        fn is_restore_err(&self) -> bool;
    }

    impl<T> ResultExt<T> for Result<T, crate::error::AppError> {
        fn is_restore_err(&self) -> bool {
            self.is_err()
        }
    }
}
