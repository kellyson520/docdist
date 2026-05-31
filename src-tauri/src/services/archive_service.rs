use crate::db::{self, Archive, DbPool};
use crate::diff;
use crate::diff::types::*;
use crate::diff::DiffStats as DiffResultStats;
use crate::error::AppError;
use crate::storage;
use std::collections::VecDeque;
use std::path::{Component, Path};

/// 验证归档源文件路径安全性（防信息泄露 CWE-200）
/// 规范化路径（解析符号链接），拒绝系统敏感目录
fn validate_file_path(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Err(AppError::Other("文件不存在".to_string()));
    }

    // 规范化路径（解析符号链接、. 和 .. 等）
    let canonical = path
        .canonicalize()
        .map_err(|_| AppError::Other("无法解析文件路径".to_string()))?;

    let path_str = canonical.to_string_lossy().to_string();
    let forbidden = [
        "/etc", "/sys", "/proc", "/dev", "/root", "/boot", "/var/log",
    ];
    for prefix in &forbidden {
        if path_str == *prefix || path_str.starts_with(&format!("{}/", prefix))
        {
            return Err(AppError::Other(format!(
                "不允许归档系统文件: {}",
                prefix
            )));
        }
    }

    Ok(())
}

/// 验证还原目标路径安全性（拒绝路径遍历和系统敏感目录）
fn validate_target_path(path: &Path) -> Result<(), AppError> {
    // 1. 检查路径组件中没有 ..
    for component in path.components() {
        if let Component::ParentDir = component {
            return Err(AppError::Other("路径不允许包含 ..".to_string()));
        }
    }

    // 2. 检查不在系统敏感目录
    let path_str = path.to_string_lossy().to_string();
    let forbidden = ["/etc", "/sys", "/proc", "/dev", "/root", "/boot"];
    for prefix in &forbidden {
        if path_str.starts_with(prefix) {
            return Err(AppError::Other(format!(
                "不允许写入系统目录: {}",
                prefix
            )));
        }
    }

    Ok(())
}

pub struct ArchiveService {
    pool: DbPool,
    chunks_dir: std::path::PathBuf,
    chunk_size: usize,
}

impl ArchiveService {
    pub fn new(pool: DbPool, data_dir: &Path, chunk_size: usize) -> Self {
        let chunks_dir = data_dir.join("chunks");
        std::fs::create_dir_all(&chunks_dir).ok();
        Self {
            pool,
            chunks_dir,
            chunk_size,
        }
    }

    /// 创建存档 — 核心功能（事务保护）
    pub fn create_archive(
        &self,
        file_path: &str,
        note: &str,
        tags: Vec<String>,
        parent_id: Option<String>,
    ) -> Result<Archive, AppError> {
        let path = Path::new(file_path);
        validate_file_path(path)?;

        // 分块存储文件（使用配置的 chunk_size）
        let (chunk_hashes, chunk_sizes, file_size, checksum) =
            storage::store_file(&self.chunks_dir, path, self.chunk_size)?;

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 自动查找父存档（同一文件的最新存档）
        let actual_parent = if parent_id.is_none() {
            let timeline = db::get_timeline(&self.pool, file_path)?;
            timeline.first().map(|a| a.id.clone())
        } else {
            parent_id
        };

        // 循环引用检测：遍历 parent_id 链，确保不会形成环
        if let Some(ref pid) = actual_parent {
            if self.has_circular_reference(pid)? {
                return Err(AppError::Other("检测到循环引用".to_string()));
            }
        }

        let archive = Archive {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: file_path.to_string(),
            file_name,
            file_size: file_size as i64,
            checksum,
            chunk_count: chunk_hashes.len() as i64,
            note: note.to_string(),
            tags,
            parent_id: actual_parent,
            created_at: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        };

        // 事务保护：chunk upsert + archive_chunks 插入 + archive 插入
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        for (i, hash) in chunk_hashes.iter().enumerate() {
            let size = chunk_sizes[i];
            let storage_path = format!("{}/{}", &hash[..2], hash);
            tx.execute(
                "INSERT INTO chunks (hash, size, ref_count, storage_path) \
                 VALUES (?1, ?2, 1, ?3) \
                 ON CONFLICT(hash) DO UPDATE SET ref_count = ref_count + 1",
                rusqlite::params![hash, size as i64, storage_path],
            )?;
        }

        for (i, hash) in chunk_hashes.iter().enumerate() {
            tx.execute(
                "INSERT OR IGNORE INTO archive_chunks \
                 (archive_id, chunk_hash, chunk_index) \
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![archive.id, hash, i as i64],
            )?;
        }

        let tags_json = serde_json::to_string(&archive.tags)?;
        tx.execute(
            "INSERT INTO archives (\
                 id, file_path, file_name, file_size, \
                 checksum, chunk_count, note, tags, \
                 parent_id, created_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                archive.id,
                archive.file_path,
                archive.file_name,
                archive.file_size,
                archive.checksum,
                archive.chunk_count,
                archive.note,
                tags_json,
                archive.parent_id,
                archive.created_at
            ],
        )?;

        tx.commit()?;

        tracing::info!(
            "Archive created: {} ({}, {} chunks)",
            archive.id,
            file_path,
            chunk_hashes.len()
        );

        Ok(archive)
    }

    /// 恢复存档到指定路径
    pub fn restore_archive(
        &self,
        archive_id: &str,
        target_path: Option<&str>,
    ) -> Result<(), AppError> {
        let archive = db::get_archive(&self.pool, archive_id)?
            .ok_or_else(|| AppError::Other("存档不存在".to_string()))?;

        let chunk_hashes = db::get_archive_chunks(&self.pool, archive_id)?;

        let output_path = match target_path {
            Some(p) => std::path::PathBuf::from(p),
            None => std::path::PathBuf::from(&archive.file_path),
        };

        if target_path.is_none() && !output_path.exists() {
            tracing::warn!(
                "原文件已不存在，将恢复创建新文件: {}",
                output_path.display()
            );
        }

        validate_target_path(&output_path)?;

        storage::restore_file(&self.chunks_dir, &chunk_hashes, &output_path)?;

        tracing::info!(
            "Archive restored: {} -> {}",
            archive_id,
            output_path.display()
        );

        Ok(())
    }

    /// 列出存档
    pub fn list_archives(
        &self,
        file_path: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<Archive>, AppError> {
        db::get_archives(&self.pool, file_path, search)
    }

    /// 分页查询存档
    pub fn list_archives_paginated(
        &self,
        file_path: Option<&str>,
        search: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Archive>, i64), AppError> {
        db::get_archives_paginated(
            &self.pool, file_path, search, page, page_size,
        )
    }

    /// 删除存档 — 事务保护
    pub fn delete_archive(&self, archive_id: &str) -> Result<(), AppError> {
        // 先获取该存档的 chunks
        let chunk_hashes = db::get_archive_chunks(&self.pool, archive_id)?;

        // 事务保护：删除关联 + 删除记录 + 减少引用计数
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "DELETE FROM archive_chunks WHERE archive_id = ?1",
            rusqlite::params![archive_id],
        )?;
        tx.execute(
            "DELETE FROM archives WHERE id = ?1",
            rusqlite::params![archive_id],
        )?;

        for hash in &chunk_hashes {
            tx.execute(
                "UPDATE chunks SET ref_count = MAX(0, ref_count - 1) WHERE hash = ?1",
                rusqlite::params![hash],
            )?;
        }

        tx.commit()?;

        tracing::info!("Archive deleted: {}", archive_id);
        Ok(())
    }

    /// 批量删除存档（单个事务保护，IN 子句批量操作，避免 N+1 查询）
    pub fn delete_archives_batch(
        &self,
        ids: &[String],
    ) -> Result<usize, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let in_clause = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let id_params: Vec<rusqlite::types::Value> = ids
            .iter()
            .map(|id| rusqlite::types::Value::Text(id.clone()))
            .collect();

        // Step 1: 一次性获取所有 archive_chunks 关联（300+ 次查询 → 1 次）
        let mut stmt = tx.prepare(&format!(
            "SELECT chunk_hash FROM archive_chunks WHERE archive_id IN ({})",
            in_clause
        ))?;
        let all_chunk_hashes: Vec<String> = stmt
            .query_map(rusqlite::params_from_iter(id_params.iter()), |row| {
                row.get::<_, String>(0)
            })?
            .map(|r| r.map_err(crate::error::AppError::Db))
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        // Step 2: 一次性删除 archive_chunks（100 次 DELETE → 1 次）
        let sql = format!(
            "DELETE FROM archive_chunks WHERE archive_id IN ({})",
            in_clause
        );
        tx.execute(&sql, rusqlite::params_from_iter(id_params.iter()))?;

        // Step 3: 一次性删除 archives（100 次 DELETE → 1 次）
        let sql = format!("DELETE FROM archives WHERE id IN ({})", in_clause);
        let total_deleted =
            tx.execute(&sql, rusqlite::params_from_iter(id_params.iter()))?;

        // Step 4: 批量减少 chunk ref_count（去重后逐 hash 更新）
        let mut unique_hashes: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for hash in &all_chunk_hashes {
            unique_hashes.insert(hash.clone());
        }
        for hash in &unique_hashes {
            tx.execute(
                "UPDATE chunks SET ref_count = MAX(0, ref_count - 1) \
                 WHERE hash = ?1",
                rusqlite::params![hash],
            )?;
        }

        tx.commit()?;
        Ok(total_deleted)
    }

    /// 更新存档备注和标签
    pub fn update_archive(
        &self,
        archive_id: &str,
        note: &str,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        db::update_archive(&self.pool, archive_id, note, &tags)
    }

    /// 对比两个存档
    pub fn compare_archives(
        &self,
        id1: &str,
        id2: &str,
    ) -> Result<diff::DiffResult, AppError> {
        let chunks1 = db::get_archive_chunks(&self.pool, id1)?;
        let chunks2 = db::get_archive_chunks(&self.pool, id2)?;

        let text1 = self.read_chunks_as_text(&chunks1)?;
        let text2 = self.read_chunks_as_text(&chunks2)?;

        Ok(diff::compute_diff(&text1, &text2))
    }

    /// 获取文件的时间线
    pub fn get_timeline(
        &self,
        file_path: &str,
    ) -> Result<Vec<Archive>, AppError> {
        db::get_timeline(&self.pool, file_path)
    }

    /// 获取子存档
    pub fn get_children(
        &self,
        parent_id: &str,
    ) -> Result<Vec<Archive>, AppError> {
        db::get_children(&self.pool, parent_id)
    }

    /// 递归获取归档树（根节点 + 所有后代）
    /// 返回扁平列表，前端可直接用 buildTree 重建树结构
    pub fn get_archive_tree(
        &self,
        file_path: Option<&str>,
    ) -> Result<Vec<Archive>, AppError> {
        // 获取所有归档，按 file_path 过滤（如果有）
        let all = db::get_archives(&self.pool, file_path, None)?;

        // 找出根节点：parent_id 为 None，或者 parent_id 不在当前集合中
        let id_set: std::collections::HashSet<String> =
            all.iter().map(|a| a.id.clone()).collect();
        let roots: Vec<Archive> = all
            .into_iter()
            .filter(|a| {
                a.parent_id.is_none()
                    || !a
                        .parent_id
                        .as_ref()
                        .is_some_and(|pid| id_set.contains(pid))
            })
            .collect();

        // 迭代收集每个根节点及其所有后代（BFS，避免递归栈溢出）
        let mut result = Vec::new();
        for root in &roots {
            result.extend(self.collect_descendants_iterative(root)?);
        }

        Ok(result)
    }

    /// 迭代收集归档节点及其所有后代（BFS）
    fn collect_descendants_iterative(
        &self,
        root: &Archive,
    ) -> Result<Vec<Archive>, AppError> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(root.clone());

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            let children = db::get_children(&self.pool, &current.id)?;
            for child in children {
                queue.push_back(child);
            }
        }
        Ok(result)
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> Result<serde_json::Value, AppError> {
        let mut stats = db::get_statistics(&self.pool)?;

        // 添加存储使用信息
        let storage_usage = storage::get_storage_usage(&self.chunks_dir)?;
        stats["storage_chunks"] = serde_json::json!(storage_usage.total_chunks);
        stats["storage_bytes"] = serde_json::json!(storage_usage.total_bytes);

        Ok(stats)
    }

    /// 清理孤儿 chunks
    pub fn cleanup_orphan_chunks(
        &self,
    ) -> Result<storage::CleanupStats, AppError> {
        let active_hashes = db::get_all_chunk_hashes(&self.pool)?;
        storage::cleanup_orphan_chunks(&self.chunks_dir, &active_hashes)
    }

    /// 验证 chunk 完整性
    pub fn verify_chunks(&self) -> Result<Vec<String>, AppError> {
        let all_hashes = db::get_all_chunk_hashes(&self.pool)?;
        let mut corrupted = Vec::new();

        for hash in &all_hashes {
            if !storage::verify_chunk(&self.chunks_dir, hash)? {
                corrupted.push(hash.clone());
            }
        }

        Ok(corrupted)
    }

    /// 检测 parent_id 链是否存在循环引用
    /// 从 target_parent_id 开始沿 parent_id 向上遍历，
    /// 如果出现重复节点则说明存在环
    fn has_circular_reference(
        &self,
        target_parent_id: &str,
    ) -> Result<bool, AppError> {
        let mut current = Some(target_parent_id.to_string());
        let mut visited = std::collections::HashSet::new();

        while let Some(pid) = current {
            if visited.contains(&pid) {
                return Ok(true);
            }
            visited.insert(pid.clone());
            let parent = db::get_archive(&self.pool, &pid)?;
            current = parent.and_then(|a| a.parent_id);
        }
        Ok(false)
    }

    fn read_chunks_as_text(
        &self,
        chunk_hashes: &[String],
    ) -> Result<String, AppError> {
        let mut content = Vec::new();
        for hash in chunk_hashes {
            if hash.len() < 2 {
                tracing::warn!("Skipping chunk with invalid hash: {}", hash);
                continue;
            }
            let chunk_path = self.chunks_dir.join(&hash[..2]).join(hash);
            if chunk_path.exists() {
                let data = std::fs::read(&chunk_path)?;
                content.extend_from_slice(&data);
            }
        }
        Ok(String::from_utf8_lossy(&content).to_string())
    }

    /// 读取 chunks 为原始字节
    fn read_chunks_as_bytes(
        &self,
        chunk_hashes: &[String],
    ) -> Result<Vec<u8>, AppError> {
        let mut content = Vec::new();
        for hash in chunk_hashes {
            if hash.len() < 2 {
                tracing::warn!("Skipping chunk with invalid hash: {}", hash);
                continue;
            }
            let chunk_path = self.chunks_dir.join(&hash[..2]).join(hash);
            if chunk_path.exists() {
                let data = std::fs::read(&chunk_path)?;
                content.extend_from_slice(&data);
            }
        }
        Ok(content)
    }

    /// 增强差异对比
    pub fn compare_archives_enhanced(
        &self,
        id1: &str,
        id2: &str,
    ) -> Result<EnhancedDiffResult, AppError> {
        // 1. 获取两个存档
        let archive1 = db::get_archive(&self.pool, id1)?
            .ok_or_else(|| AppError::Other("存档1不存在".to_string()))?;
        let _archive2 = db::get_archive(&self.pool, id2)?
            .ok_or_else(|| AppError::Other("存档2不存在".to_string()))?;

        // 2. 获取 chunks
        let chunks1 = db::get_archive_chunks(&self.pool, id1)?;
        let chunks2 = db::get_archive_chunks(&self.pool, id2)?;

        // 3. 恢复文件内容
        let old_content = self.read_chunks_as_bytes(&chunks1)?;
        let new_content = self.read_chunks_as_bytes(&chunks2)?;

        // 4. 检测文件类型
        let file_path = &archive1.file_path;
        let file_type = detect_file_type(file_path, &old_content);

        // 5. 根据类型执行差异计算
        match &file_type {
            FileType::Text { .. } => {
                // 文本文件：完整差异
                let old_text = String::from_utf8_lossy(&old_content);
                let new_text = String::from_utf8_lossy(&new_content);

                let diff_result = diff::compute_diff(&old_text, &new_text);
                let summary_gen = crate::diff::summary::SummaryGenerator::new();
                let summary = summary_gen.generate(&diff_result);

                Ok(EnhancedDiffResult {
                    diff_result,
                    summary,
                    file_type,
                    preview: Some(ContentPreview {
                        old_preview: old_text.chars().take(500).collect(),
                        new_preview: new_text.chars().take(500).collect(),
                        preview_lines: 20,
                    }),
                })
            }
            _ => {
                // 二进制文件：简单比对
                let binary_diff =
                    crate::diff::engine::BinaryDiffEngine::compare(
                        &old_content,
                        &new_content,
                    )?;

                Ok(EnhancedDiffResult {
                    diff_result: diff::DiffResult {
                        hunks: vec![],
                        stats: DiffResultStats {
                            additions: 0,
                            deletions: 0,
                            unchanged: 0,
                        },
                    },
                    summary: DiffSummary {
                        stats: DiffStats {
                            additions: 0,
                            deletions: 0,
                            modifications: 0,
                            unchanged: 0,
                            total: 0,
                        },
                        changes: vec![],
                        change_distribution: ChangeDistribution {
                            additions: 0,
                            deletions: 0,
                            modifications: 0,
                            moves: 0,
                            renames: 0,
                        },
                        affected_regions: vec![],
                        ai_summary: Some(binary_diff.summary),
                    },
                    file_type,
                    preview: None,
                })
            }
        }
    }
}

fn detect_file_type(path: &str, data: &[u8]) -> FileType {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "json" | "xml" | "html" | "css" | "js" | "ts"
        | "jsx" | "tsx" | "py" | "rs" | "go" | "java" | "c" | "cpp" | "h"
        | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" => FileType::Text {
            encoding: "UTF-8".to_string(),
            line_ending: "\n".to_string(),
        },
        "pdf" => FileType::Pdf {
            page_count: 0,
            has_images: false,
        },
        "dxf" | "dwg" => FileType::Cad {
            format: ext.to_uppercase(),
            layer_count: 0,
            entity_count: 0,
        },
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" => {
            FileType::Image {
                width: 0,
                height: 0,
                format: ext,
            }
        }
        _ => FileType::Binary {
            mime_type: "application/octet-stream".to_string(),
            size: data.len() as u64,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// 创建测试用 ArchiveService 实例，使用临时目录和文件数据库
    fn setup_service() -> (ArchiveService, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = crate::db::init_database(&db_path)
            .expect("Failed to init test database");
        let chunk_size = 4096usize;
        let service = ArchiveService::new(pool, dir.path(), chunk_size);
        (service, dir)
    }

    /// 在临时目录中创建测试文件并返回路径
    fn create_test_file(
        dir: &tempfile::TempDir,
        name: &str,
        content: &[u8],
    ) -> String {
        let file_path = dir.path().join(name);
        fs::write(&file_path, content).unwrap();
        file_path.to_string_lossy().to_string()
    }

    // ================================================================
    // Test 1: create_archive 正常文件 → 返回 Archive，chunks 已写入磁盘
    // ================================================================
    #[test]
    fn test_create_archive_normal_file() {
        let (service, dir) = setup_service();
        let file_path = create_test_file(&dir, "test.txt", b"Hello, DocDist!");

        let result = service.create_archive(
            &file_path,
            "test note",
            vec!["tag1".to_string(), "tag2".to_string()],
            None,
        );

        let archive = result.expect("create_archive should succeed");
        assert!(!archive.id.is_empty(), "archive id should not be empty");
        assert_eq!(archive.file_path, file_path);
        assert_eq!(archive.file_name, "test.txt");
        assert_eq!(archive.file_size, 15); // "Hello, DocDist!" = 15 bytes
        assert!(!archive.checksum.is_empty());
        assert!(archive.chunk_count >= 1);
        assert_eq!(archive.note, "test note");
        assert_eq!(archive.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert!(
            archive.parent_id.is_none(),
            "first archive should have no parent"
        );
        assert!(!archive.created_at.is_empty());

        // 验证 chunks 已写入磁盘
        let chunks_dir = dir.path().join("chunks");
        assert!(chunks_dir.exists(), "chunks directory should exist");
        // 至少有一个子目录（hash 前两位）
        let subdirs: Vec<_> = fs::read_dir(&chunks_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect();
        assert!(
            !subdirs.is_empty(),
            "should have at least one chunk subdirectory"
        );
    }

    // ================================================================
    // Test 2: create_archive 不存在文件 → 返回 AppError
    // ================================================================
    #[test]
    fn test_create_archive_nonexistent_file() {
        let (service, _dir) = setup_service();

        let result = service.create_archive(
            "/nonexistent/path/file.txt",
            "note",
            vec![],
            None,
        );

        assert!(result.is_err(), "should return error for nonexistent file");
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("文件不存在"),
            "error message should contain '文件不存在', got: {}",
            msg
        );
    }

    // ================================================================
    // Test 3: create_archive 自动 parent_id 链（同一文件多次创建）
    // ================================================================
    #[test]
    fn test_create_archive_auto_parent_chain() {
        let (service, dir) = setup_service();
        let file_path = create_test_file(&dir, "chain.txt", b"version 1");

        // 第一次创建 — 没有 parent
        let archive1 = service
            .create_archive(&file_path, "v1", vec![], None)
            .unwrap();
        assert!(
            archive1.parent_id.is_none(),
            "first archive should have no parent"
        );

        // 修改文件内容，等待1.1秒确保 created_at 不同（精度为秒）
        fs::write(&file_path, b"version 2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));

        // 第二次创建 — 应自动链接 parent
        let archive2 = service
            .create_archive(&file_path, "v2", vec![], None)
            .unwrap();
        assert_eq!(
            archive2.parent_id,
            Some(archive1.id.clone()),
            "second archive's parent should be the first archive"
        );

        // 修改文件内容，等待1.1秒
        fs::write(&file_path, b"version 3").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));

        // 第三次创建 — parent 应为第二次
        let archive3 = service
            .create_archive(&file_path, "v3", vec![], None)
            .unwrap();
        assert_eq!(
            archive3.parent_id,
            Some(archive2.id.clone()),
            "third archive's parent should be the second archive"
        );
    }

    // ================================================================
    // Test 4: restore_archive 还原到原始路径
    // ================================================================
    #[test]
    fn test_restore_archive_to_original_path() {
        let (service, dir) = setup_service();
        let original_content = b"original content for restore test";
        let file_path =
            create_test_file(&dir, "restore_orig.txt", original_content);

        let archive = service
            .create_archive(&file_path, "for restore", vec![], None)
            .unwrap();

        // 删除原始文件
        fs::remove_file(&file_path).unwrap();
        assert!(!Path::new(&file_path).exists());

        // 恢复到原始路径
        service.restore_archive(&archive.id, None).unwrap();

        // 验证恢复的文件内容
        assert!(Path::new(&file_path).exists(), "file should be restored");
        let restored = fs::read(&file_path).unwrap();
        assert_eq!(
            restored, original_content,
            "restored content should match original"
        );
    }

    // ================================================================
    // Test 5: restore_archive 还原到自定义路径
    // ================================================================
    #[test]
    fn test_restore_archive_to_custom_path() {
        let (service, dir) = setup_service();
        let original_content = b"custom path restore test content";
        let file_path =
            create_test_file(&dir, "restore_custom.txt", original_content);

        let archive = service
            .create_archive(&file_path, "for custom restore", vec![], None)
            .unwrap();

        // 自定义恢复路径
        let custom_path = dir.path().join("subdir").join("restored_file.txt");
        let custom_path_str = custom_path.to_string_lossy().to_string();

        service
            .restore_archive(&archive.id, Some(&custom_path_str))
            .unwrap();

        assert!(
            custom_path.exists(),
            "file should be restored to custom path"
        );
        let restored = fs::read(&custom_path).unwrap();
        assert_eq!(
            restored, original_content,
            "restored content should match original"
        );
    }

    // ================================================================
    // Test 6: restore_archive 不存在 id → 返回错误
    // ================================================================
    #[test]
    fn test_restore_archive_nonexistent_id() {
        let (service, _dir) = setup_service();

        let result = service.restore_archive("nonexistent-id-12345", None);

        assert!(
            result.is_err(),
            "should return error for nonexistent archive id"
        );
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("存档不存在"),
            "error message should contain '存档不存在', got: {}",
            msg
        );
    }

    // ================================================================
    // Test 7: delete_archive 清理 chunks 引用计数
    // ================================================================
    #[test]
    fn test_delete_archive_ref_count_cleanup() {
        let (service, dir) = setup_service();
        let file_path =
            create_test_file(&dir, "delete_ref.txt", b"ref count test");

        let archive = service
            .create_archive(&file_path, "for delete", vec![], None)
            .unwrap();

        // 验证存档存在
        let archives = service.list_archives(None, None).unwrap();
        assert_eq!(archives.len(), 1);

        // 删除存档
        service.delete_archive(&archive.id).unwrap();

        // 验证存档已删除
        let archives = service.list_archives(None, None).unwrap();
        assert_eq!(archives.len(), 0, "archive should be deleted");

        // 验证 chunks 引用计数已减少（chunks 表中 ref_count 应为 0）
        let conn = service.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT ref_count FROM chunks").unwrap();
        let counts: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for count in &counts {
            assert_eq!(
                *count, 0,
                "ref_count should be 0 after deletion, got {}",
                count
            );
        }
    }

    // ================================================================
    // Test 8: delete_archives_batch 事务保护
    // ================================================================
    #[test]
    fn test_delete_archives_batch_transaction() {
        let (service, dir) = setup_service();

        // 创建多个存档
        let fp1 = create_test_file(&dir, "batch1.txt", b"batch content 1");
        let fp2 = create_test_file(&dir, "batch2.txt", b"batch content 2");
        let fp3 = create_test_file(&dir, "batch3.txt", b"batch content 3");

        let a1 = service.create_archive(&fp1, "a1", vec![], None).unwrap();
        let a2 = service.create_archive(&fp2, "a2", vec![], None).unwrap();
        let a3 = service.create_archive(&fp3, "a3", vec![], None).unwrap();

        // 确认有 3 个存档
        let archives = service.list_archives(None, None).unwrap();
        assert_eq!(archives.len(), 3);

        // 批量删除 a1 和 a2
        let ids = vec![a1.id.clone(), a2.id.clone()];
        let deleted = service.delete_archives_batch(&ids).unwrap();
        assert_eq!(deleted, 2, "should delete 2 archives");

        // 确认只剩 a3
        let archives = service.list_archives(None, None).unwrap();
        assert_eq!(archives.len(), 1);
        assert_eq!(archives[0].id, a3.id);

        // 删除空列表不应失败
        let deleted = service.delete_archives_batch(&[]).unwrap();
        assert_eq!(deleted, 0, "deleting empty list should return 0");
    }

    // ================================================================
    // Test 9: compare_archives 两个存档 diff 结果正确
    // ================================================================
    #[test]
    fn test_compare_archives_diff_result() {
        let (service, dir) = setup_service();

        // 创建两个不同内容的存档
        let fp =
            create_test_file(&dir, "diff.txt", b"line 1\nline 2\nline 3\n");
        let a1 = service.create_archive(&fp, "v1", vec![], None).unwrap();

        fs::write(&fp, b"line 1\nmodified line\nline 3\nnew line\n").unwrap();
        let a2 = service.create_archive(&fp, "v2", vec![], None).unwrap();

        let diff = service.compare_archives(&a1.id, &a2.id).unwrap();

        // 验证 diff 结构合理
        // "line 1" 和 "line 3" 不变，"line 2" 被删除，"modified line" 和 "new line" 被添加
        assert!(diff.stats.additions > 0, "should have additions");
        assert!(diff.stats.deletions > 0, "should have deletions");
        assert!(diff.stats.unchanged > 0, "should have unchanged lines");
        assert!(!diff.hunks.is_empty(), "should have at least one hunk");
    }

    // ================================================================
    // Test 10: list_archives / list_archives_paginated 正确
    // ================================================================
    #[test]
    fn test_list_archives_and_paginated() {
        let (service, dir) = setup_service();

        // 创建 5 个存档
        for i in 0..5 {
            let fp = create_test_file(
                &dir,
                &format!("list_{}.txt", i),
                format!("content {}", i).as_bytes(),
            );
            service
                .create_archive(&fp, &format!("note {}", i), vec![], None)
                .unwrap();
        }

        // list_archives 全部
        let all = service.list_archives(None, None).unwrap();
        assert_eq!(all.len(), 5, "should list all 5 archives");

        // list_archives 按 file_path 过滤
        let fp0 = dir.path().join("list_0.txt").to_string_lossy().to_string();
        let filtered = service.list_archives(Some(&fp0), None).unwrap();
        assert_eq!(filtered.len(), 1, "should filter by file_path");

        // list_archives 按 search 过滤
        let searched = service.list_archives(None, Some("list_2")).unwrap();
        assert_eq!(searched.len(), 1, "should search by file_name");

        // list_archives_paginated 第 1 页
        let (page1, total) =
            service.list_archives_paginated(None, None, 1, 2).unwrap();
        assert_eq!(total, 5, "total should be 5");
        assert_eq!(page1.len(), 2, "page 1 should have 2 items");

        // list_archives_paginated 第 2 页
        let (page2, total2) =
            service.list_archives_paginated(None, None, 2, 2).unwrap();
        assert_eq!(total2, 5);
        assert_eq!(page2.len(), 2, "page 2 should have 2 items");

        // list_archives_paginated 第 3 页（只有 1 项）
        let (page3, total3) =
            service.list_archives_paginated(None, None, 3, 2).unwrap();
        assert_eq!(total3, 5);
        assert_eq!(page3.len(), 1, "page 3 should have 1 item");

        // 验证 ID 不重复
        let mut all_ids: Vec<&str> = page1
            .iter()
            .chain(page2.iter())
            .chain(page3.iter())
            .map(|a| a.id.as_str())
            .collect();
        all_ids.sort();
        all_ids.dedup();
        assert_eq!(
            all_ids.len(),
            5,
            "all paginated items should have unique IDs"
        );
    }

    // ================================================================
    // Test 11: get_timeline 返回同一文件的时间序列
    // ================================================================
    #[test]
    fn test_get_timeline_same_file() {
        let (service, dir) = setup_service();
        let fp = create_test_file(&dir, "timeline.txt", b"v1");

        let a1 = service.create_archive(&fp, "v1", vec![], None).unwrap();

        fs::write(&fp, b"v2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let a2 = service.create_archive(&fp, "v2", vec![], None).unwrap();

        fs::write(&fp, b"v3").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let a3 = service.create_archive(&fp, "v3", vec![], None).unwrap();

        let timeline = service.get_timeline(&fp).unwrap();

        assert_eq!(timeline.len(), 3, "timeline should have 3 entries");
        // timeline 按 created_at DESC 排序，所以最新的在前
        assert_eq!(timeline[0].id, a3.id, "newest should be first");
        assert_eq!(timeline[1].id, a2.id);
        assert_eq!(timeline[2].id, a1.id, "oldest should be last");

        // 不同文件的 timeline 应为空
        let fp2 = dir.path().join("other.txt").to_string_lossy().to_string();
        let other_timeline = service.get_timeline(&fp2).unwrap();
        assert_eq!(
            other_timeline.len(),
            0,
            "different file should have empty timeline"
        );
    }

    // ================================================================
    // Test 12: get_children 返回子存档
    // ================================================================
    #[test]
    fn test_get_children() {
        let (service, dir) = setup_service();
        let fp = create_test_file(&dir, "children.txt", b"parent content");

        // 创建父存档
        let parent =
            service.create_archive(&fp, "parent", vec![], None).unwrap();

        // 创建子存档（显式指定 parent_id）
        fs::write(&fp, b"child 1 content").unwrap();
        let child1 = service
            .create_archive(&fp, "child1", vec![], Some(parent.id.clone()))
            .unwrap();

        fs::write(&fp, b"child 2 content").unwrap();
        let child2 = service
            .create_archive(&fp, "child2", vec![], Some(parent.id.clone()))
            .unwrap();

        let children = service.get_children(&parent.id).unwrap();

        assert_eq!(children.len(), 2, "parent should have 2 children");
        let child_ids: Vec<&str> =
            children.iter().map(|a| a.id.as_str()).collect();
        assert!(child_ids.contains(&child1.id.as_str()));
        assert!(child_ids.contains(&child2.id.as_str()));

        // 无子存档的 ID 应返回空列表
        let no_children =
            service.get_children("nonexistent-parent-id").unwrap();
        assert_eq!(no_children.len(), 0);
    }

    // ================================================================
    // Test 13: cleanup_orphan_chunks 正确清理
    // ================================================================
    #[test]
    fn test_cleanup_orphan_chunks() {
        let (service, dir) = setup_service();

        // 创建一个存档（会写入 chunks）
        let fp = create_test_file(
            &dir,
            "orphan_test.txt",
            b"orphan cleanup content",
        );
        let _archive = service
            .create_archive(&fp, "orphan test", vec![], None)
            .unwrap();

        // 手动在 chunks 目录下创建一个孤儿 chunk 文件
        let chunks_dir = dir.path().join("chunks");
        let orphan_prefix = chunks_dir.join("ff");
        fs::create_dir_all(&orphan_prefix).unwrap();
        let orphan_hash = format!("ff{}", "0".repeat(14));
        fs::write(orphan_prefix.join(&orphan_hash), b"orphan data").unwrap();

        // 确认孤儿文件存在
        assert!(
            orphan_prefix.join(&orphan_hash).exists(),
            "orphan file should exist before cleanup"
        );

        // 执行清理
        let stats = service.cleanup_orphan_chunks().unwrap();

        // 孤儿 chunk 应被删除
        assert!(
            stats.removed_count >= 1,
            "should remove at least 1 orphan chunk, got {}",
            stats.removed_count
        );
        assert!(
            !orphan_prefix.join(&orphan_hash).exists(),
            "orphan file should be removed after cleanup"
        );

        // 活跃的 chunk 应被保留
        assert!(
            stats.kept_count >= 1,
            "should keep at least 1 active chunk, got {}",
            stats.kept_count
        );
    }

    // ================================================================
    // Test 14: verify_chunks 检测损坏 chunk
    // ================================================================
    #[test]
    fn test_verify_chunks_detects_corruption() {
        let (service, dir) = setup_service();

        // 创建一个存档
        let fp =
            create_test_file(&dir, "verify.txt", b"verify chunk integrity");
        let _archive = service
            .create_archive(&fp, "verify test", vec![], None)
            .unwrap();

        // 正常状态下不应有损坏
        let corrupted = service.verify_chunks().unwrap();
        assert!(
            corrupted.is_empty(),
            "should have no corrupted chunks initially, got {:?}",
            corrupted
        );

        // 获取该存档的 chunk hash，然后篡改 chunk 文件内容
        let conn = service.pool.get().unwrap();
        let chunk_hash: String = conn
            .query_row(
                "SELECT chunk_hash FROM archive_chunks LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let prefix = &chunk_hash[..2];
        let chunk_path =
            dir.path().join("chunks").join(prefix).join(&chunk_hash);
        assert!(chunk_path.exists(), "chunk file should exist");

        // 篡改 chunk 内容
        fs::write(&chunk_path, b"TAMPERED DATA!").unwrap();

        // 验证应检测到损坏
        let corrupted = service.verify_chunks().unwrap();
        assert_eq!(corrupted.len(), 1, "should detect 1 corrupted chunk");
        assert_eq!(corrupted[0], chunk_hash, "should report the tampered hash");
    }

    // ================================================================
    // 额外测试: create_archive 多块文件
    // ================================================================
    #[test]
    fn test_create_archive_multi_chunk_file() {
        let (service, dir) = setup_service();
        // 10000 字节，chunk_size=4096，应分为 3 个 chunk
        let content: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let fp = create_test_file(&dir, "large.bin", &content);

        let archive =
            service.create_archive(&fp, "large", vec![], None).unwrap();
        assert_eq!(archive.file_size, 10000);
        assert_eq!(
            archive.chunk_count, 3,
            "10000 bytes with chunk_size=4096 should produce 3 chunks"
        );
    }

    // ================================================================
    // 额外测试: update_archive
    // ================================================================
    #[test]
    fn test_update_archive_note_and_tags() {
        let (service, dir) = setup_service();
        let fp = create_test_file(&dir, "update.txt", b"update test");

        let archive = service
            .create_archive(&fp, "original note", vec!["old".to_string()], None)
            .unwrap();

        service
            .update_archive(
                &archive.id,
                "updated note",
                vec!["new1".to_string(), "new2".to_string()],
            )
            .unwrap();

        let updated = service.list_archives(None, None).unwrap();
        let found = updated.iter().find(|a| a.id == archive.id).unwrap();
        assert_eq!(found.note, "updated note");
        assert_eq!(found.tags, vec!["new1".to_string(), "new2".to_string()]);
    }

    // ================================================================
    // 额外测试: get_statistics
    // ================================================================
    #[test]
    fn test_get_statistics() {
        let (service, dir) = setup_service();

        // 空数据库
        let stats = service.get_statistics().unwrap();
        assert_eq!(stats["total_archives"], 0);

        // 创建存档后统计
        let fp = create_test_file(&dir, "stats.txt", b"stats content");
        service
            .create_archive(&fp, "stats test", vec![], None)
            .unwrap();

        let stats = service.get_statistics().unwrap();
        assert_eq!(stats["total_archives"], 1);
        assert!(stats["total_size"].as_i64().unwrap() > 0);
        assert_eq!(stats["unique_files"], 1);
        assert!(stats["storage_chunks"].as_u64().unwrap() >= 1);
        assert!(stats["storage_bytes"].as_u64().unwrap() > 0);
    }

    // ================================================================
    // 额外测试: 空文件的存档和恢复
    // ================================================================
    #[test]
    fn test_create_and_restore_empty_file() {
        let (service, dir) = setup_service();
        let fp = create_test_file(&dir, "empty.txt", b"");

        let archive = service
            .create_archive(&fp, "empty file", vec![], None)
            .unwrap();
        assert_eq!(archive.file_size, 0);

        let restore_path = dir
            .path()
            .join("restored_empty.txt")
            .to_string_lossy()
            .to_string();
        service
            .restore_archive(&archive.id, Some(&restore_path))
            .unwrap();
        let restored = fs::read(&restore_path).unwrap();
        assert!(
            restored.is_empty(),
            "restored empty file should still be empty"
        );
    }

    // ================================================================
    // 额外测试: 多个文件各自独立的 parent 链
    // ================================================================
    #[test]
    fn test_multiple_files_independent_chains() {
        let (service, dir) = setup_service();
        let fp1 = create_test_file(&dir, "chain_a.txt", b"a1");
        let fp2 = create_test_file(&dir, "chain_b.txt", b"b1");

        let a1 = service.create_archive(&fp1, "a1", vec![], None).unwrap();
        let b1 = service.create_archive(&fp2, "b1", vec![], None).unwrap();

        assert!(a1.parent_id.is_none());
        assert!(
            b1.parent_id.is_none(),
            "different files should have independent chains"
        );

        fs::write(&fp1, b"a2").unwrap();
        let a2 = service.create_archive(&fp1, "a2", vec![], None).unwrap();
        assert_eq!(a2.parent_id, Some(a1.id.clone()));

        // b1 的 parent 链不受影响
        let timeline_b = service.get_timeline(&fp2).unwrap();
        assert_eq!(timeline_b.len(), 1);
    }

    // ================================================================
    // 额外测试: delete_archives_batch 事务回滚语义
    // ================================================================
    #[test]
    fn test_delete_batch_all_ids() {
        let (service, dir) = setup_service();
        let mut ids = Vec::new();
        for i in 0..10 {
            let fp = create_test_file(
                &dir,
                &format!("batch_del_{}.txt", i),
                format!("c{}", i).as_bytes(),
            );
            let a = service
                .create_archive(&fp, &format!("bd{}", i), vec![], None)
                .unwrap();
            ids.push(a.id);
        }

        let all = service.list_archives(None, None).unwrap();
        assert_eq!(all.len(), 10);

        let deleted = service.delete_archives_batch(&ids).unwrap();
        assert_eq!(deleted, 10);

        let remaining = service.list_archives(None, None).unwrap();
        assert_eq!(remaining.len(), 0, "all archives should be deleted");
    }

    // ================================================================
    // 额外测试: create_archive 自动 parent 不跨文件
    // ================================================================
    #[test]
    fn test_create_archive_no_auto_parent_for_new_file() {
        let (service, dir) = setup_service();
        let fp1 = create_test_file(&dir, "no_cross.txt", b"content");

        let a1 = service.create_archive(&fp1, "first", vec![], None).unwrap();
        assert!(a1.parent_id.is_none());

        // 不同文件名但可能在同一目录
        let fp2 = create_test_file(&dir, "no_cross_2.txt", b"other content");
        let a2 = service
            .create_archive(&fp2, "second", vec![], None)
            .unwrap();
        assert!(
            a2.parent_id.is_none(),
            "different file should not have auto parent"
        );
    }

    // ================================================================
    // 额外测试: get_archive_tree 递归返回完整树
    // ================================================================
    #[test]
    fn test_get_archive_tree_returns_full_tree() {
        let (service, dir) = setup_service();
        let fp = create_test_file(&dir, "tree.txt", b"v1");

        // 建立链: v1 -> v2 -> v3
        let a1 = service.create_archive(&fp, "v1", vec![], None).unwrap();
        fs::write(&fp, b"v2").unwrap();
        let a2 = service.create_archive(&fp, "v2", vec![], None).unwrap();
        fs::write(&fp, b"v3").unwrap();
        let a3 = service.create_archive(&fp, "v3", vec![], None).unwrap();

        // get_archive_tree 应返回所有 3 个节点
        let tree = service.get_archive_tree(None).unwrap();
        assert_eq!(tree.len(), 3, "tree should contain all 3 archives");

        let ids: Vec<&str> = tree.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&a1.id.as_str()));
        assert!(ids.contains(&a2.id.as_str()));
        assert!(ids.contains(&a3.id.as_str()));

        // 按 file_path 过滤也应返回所有 3 个
        let tree_filtered = service.get_archive_tree(Some(&fp)).unwrap();
        assert_eq!(
            tree_filtered.len(),
            3,
            "filtered tree should also contain all 3"
        );
    }

    #[test]
    fn test_get_archive_tree_multiple_roots() {
        let (service, dir) = setup_service();
        let fp1 = create_test_file(&dir, "tree_a.txt", b"a1");
        let fp2 = create_test_file(&dir, "tree_b.txt", b"b1");

        let _a1 = service.create_archive(&fp1, "a1", vec![], None).unwrap();
        let _b1 = service.create_archive(&fp2, "b1", vec![], None).unwrap();

        fs::write(&fp1, b"a2").unwrap();
        let _a2 = service.create_archive(&fp1, "a2", vec![], None).unwrap();

        // 无过滤：应返回 3 个节点（2 个文件的完整树）
        let all = service.get_archive_tree(None).unwrap();
        assert_eq!(all.len(), 3, "should return all 3 archives across files");

        // 按 fp1 过滤：只返回 fp1 的 2 个节点
        let tree_a = service.get_archive_tree(Some(&fp1)).unwrap();
        assert_eq!(tree_a.len(), 2, "should return 2 archives for fp1");

        // 按 fp2 过滤：只返回 fp2 的 1 个节点
        let tree_b = service.get_archive_tree(Some(&fp2)).unwrap();
        assert_eq!(tree_b.len(), 1, "should return 1 archive for fp2");
    }

    // ================================================================
    // Test: restore_archive 路径遍历漏洞修复 (CWE-22)
    // ================================================================
    #[test]
    fn test_restore_rejects_path_traversal() {
        let result =
            validate_target_path(Path::new("/home/user/../../etc/passwd"));
        assert!(result.is_err(), "should reject path with ..");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains(".."),
            "error should mention .., got: {}",
            err_msg
        );
    }

    #[test]
    fn test_restore_rejects_system_directory() {
        let result = validate_target_path(Path::new("/etc/passwd"));
        assert!(result.is_err(), "should reject /etc path");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("不允许写入系统目录"),
            "error should mention system dir, got: {}",
            err_msg
        );

        // 验证其他敏感目录也被拒绝
        for dir in &[
            "/sys/kernel",
            "/proc/1/mem",
            "/dev/null",
            "/root/.ssh",
            "/boot/vmlinuz",
        ] {
            let result = validate_target_path(Path::new(dir));
            assert!(result.is_err(), "should reject {}", dir);
        }
    }

    #[test]
    fn test_restore_allows_normal_path() {
        let result = validate_target_path(Path::new("/home/user/file.txt"));
        assert!(
            result.is_ok(),
            "should allow normal user path, got: {:?}",
            result
        );

        let result2 = validate_target_path(Path::new("/tmp/test.txt"));
        assert!(
            result2.is_ok(),
            "should allow /tmp path, got: {:?}",
            result2
        );
    }

    // ================================================================
    // Test: create_archive 拒绝系统敏感文件 (CWE-200)
    // ================================================================
    #[test]
    fn test_create_rejects_system_file() {
        let (service, _dir) = setup_service();

        let result =
            service.create_archive("/etc/passwd", "hack", vec![], None);

        assert!(result.is_err(), "should reject /etc/passwd");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("不允许归档系统文件"),
            "error should mention forbidden path, got: {}",
            msg
        );
    }

    // ================================================================
    // Test: create_archive 允许普通用户文件
    // ================================================================
    #[test]
    fn test_create_allows_normal_file() {
        let (service, dir) = setup_service();
        let file_path = create_test_file(&dir, "normal.txt", b"safe content");

        let result = service.create_archive(&file_path, "ok", vec![], None);
        assert!(result.is_ok(), "should allow normal user file");
    }
}
