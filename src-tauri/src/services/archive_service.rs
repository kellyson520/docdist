use crate::db::{self, Archive, DbPool};
use crate::diff;
use crate::error::AppError;
use crate::storage;
use std::path::Path;

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
        if !path.exists() {
            return Err(AppError::Other("文件不存在".to_string()));
        }

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

    /// 批量删除存档（单个事务保护所有删除操作）
    pub fn delete_archives_batch(
        &self,
        ids: &[String],
    ) -> Result<usize, AppError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let mut total_deleted = 0;
        for id in ids {
            // 查询该存档关联的 chunk hashes
            let chunk_hashes: Vec<String> = {
                let mut stmt = tx.prepare(
                    "SELECT chunk_hash FROM archive_chunks \
                     WHERE archive_id = ?1 ORDER BY chunk_index",
                )?;
                let hashes: Vec<String> = stmt
                    .query_map(rusqlite::params![id], |row| {
                        row.get::<_, String>(0)
                    })?
                    .map(|r| r.map_err(crate::error::AppError::Db))
                    .collect::<Result<Vec<_>, _>>()?;
                hashes
            };

            tx.execute(
                "DELETE FROM archive_chunks WHERE archive_id = ?1",
                rusqlite::params![id],
            )?;
            let deleted = tx.execute(
                "DELETE FROM archives WHERE id = ?1",
                rusqlite::params![id],
            )?;

            for hash in &chunk_hashes {
                tx.execute(
                    "UPDATE chunks SET ref_count = MAX(0, ref_count - 1) \
                     WHERE hash = ?1",
                    rusqlite::params![hash],
                )?;
            }

            total_deleted += deleted;
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
}
