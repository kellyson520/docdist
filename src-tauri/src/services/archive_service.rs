use crate::db::{self, Archive, DbPool};
use crate::diff;
use crate::error::AppError;
use crate::storage;
use std::path::Path;

pub struct ArchiveService {
    pool: DbPool,
    chunks_dir: std::path::PathBuf,
}

impl ArchiveService {
    pub fn new(pool: DbPool, data_dir: &Path) -> Self {
        let chunks_dir = data_dir.join("chunks");
        std::fs::create_dir_all(&chunks_dir).ok();
        Self { pool, chunks_dir }
    }

    /// 创建存档 — 核心功能
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

        // 分块存储文件
        let (chunk_hashes, file_size, checksum) =
            storage::store_file(&self.chunks_dir, path)?;

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

        // 保存 chunk 信息并更新 ref_count
        let chunk_infos: Vec<(String, usize)> =
            chunk_hashes.iter().map(|h| (h.clone(), 4096)).collect();

        for (hash, size) in &chunk_infos {
            db::upsert_chunk(&self.pool, hash, *size)?;
        }

        db::insert_archive_chunks(&self.pool, &archive.id, &chunk_infos)?;
        db::insert_archive(&self.pool, &archive)?;

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
            &self.pool,
            file_path,
            search,
            page,
            page_size,
        )
    }

    /// 删除存档 — 同时清理 ref_count
    pub fn delete_archive(&self, archive_id: &str) -> Result<(), AppError> {
        // 先获取该存档的 chunks
        let chunk_hashes =
            db::get_archive_chunks(&self.pool, archive_id)?;

        // 从 archive_chunks 表删除关联
        db::delete_archive_chunks(&self.pool, archive_id)?;

        // 删除存档记录
        db::delete_archive_record(&self.pool, archive_id)?;

        // 减少 chunk 引用计数
        for hash in &chunk_hashes {
            db::decrement_chunk_ref(&self.pool, hash)?;
        }

        tracing::info!("Archive deleted: {}", archive_id);
        Ok(())
    }

    /// 批量删除存档
    pub fn delete_archives_batch(
        &self,
        ids: &[String],
    ) -> Result<usize, AppError> {
        let mut total_deleted = 0;
        for id in ids {
            self.delete_archive(id)?;
            total_deleted += 1;
        }
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
        stats["storage_chunks"] =
            serde_json::json!(storage_usage.total_chunks);
        stats["storage_bytes"] =
            serde_json::json!(storage_usage.total_bytes);

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
            let chunk_path = self.chunks_dir.join(&hash[..2]).join(hash);
            if chunk_path.exists() {
                let data = std::fs::read(&chunk_path)?;
                content.extend_from_slice(&data);
            }
        }
        Ok(String::from_utf8_lossy(&content).to_string())
    }
}
