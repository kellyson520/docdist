use std::path::Path;
use std::sync::Arc;
use crate::db::{self, Archive, DbPool};
use crate::storage;
use crate::diff;
use crate::error::AppError;

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

    pub fn create_archive(&self, file_path: &str, note: &str, tags: Vec<String>, parent_id: Option<String>) -> Result<Archive, AppError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::Other("文件不存在".to_string()));
        }

        let (chunk_hashes, file_size, checksum) = storage::store_file(&self.chunks_dir, path)?;

        // Check if content changed from latest archive
        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Find previous archive for this file to set parent_id
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
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        // Save chunks info
        let chunk_infos: Vec<(String, usize)> = chunk_hashes.iter()
            .map(|h| (h.clone(), 4096))
            .collect();
        db::insert_archive_chunks(&self.pool, &archive.id, &chunk_infos)?;

        // Save archive
        db::insert_archive(&self.pool, &archive)?;

        Ok(archive)
    }

    pub fn restore_archive(&self, archive_id: &str, target_path: Option<&str>) -> Result<(), AppError> {
        let archive = db::get_archive(&self.pool, archive_id)?
            .ok_or_else(|| AppError::Other("存档不存在".to_string()))?;

        let chunk_hashes = db::get_archive_chunks(&self.pool, archive_id)?;

        let output_path = match target_path {
            Some(p) => std::path::PathBuf::from(p),
            None => std::path::PathBuf::from(&archive.file_path),
        };

        storage::restore_file(&self.chunks_dir, &chunk_hashes, &output_path)?;
        Ok(())
    }

    pub fn list_archives(&self, file_path: Option<&str>, search: Option<&str>) -> Result<Vec<Archive>, AppError> {
        db::get_archives(&self.pool, file_path, search)
    }

    pub fn delete_archive(&self, archive_id: &str) -> Result<(), AppError> {
        db::delete_archive(&self.pool, archive_id)
    }

    pub fn update_archive(&self, archive_id: &str, note: &str, tags: Vec<String>) -> Result<(), AppError> {
        db::update_archive(&self.pool, archive_id, note, &tags)
    }

    pub fn compare_archives(&self, id1: &str, id2: &str) -> Result<diff::DiffResult, AppError> {
        let chunks1 = db::get_archive_chunks(&self.pool, id1)?;
        let chunks2 = db::get_archive_chunks(&self.pool, id2)?;

        let text1 = self.read_chunks_as_text(&chunks1)?;
        let text2 = self.read_chunks_as_text(&chunks2)?;

        Ok(diff::compute_diff(&text1, &text2))
    }

    pub fn get_timeline(&self, file_path: &str) -> Result<Vec<Archive>, AppError> {
        db::get_timeline(&self.pool, file_path)
    }

    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Archive>, AppError> {
        db::get_children(&self.pool, parent_id)
    }

    pub fn get_statistics(&self) -> Result<serde_json::Value, AppError> {
        db::get_statistics(&self.pool)
    }

    fn read_chunks_as_text(&self, chunk_hashes: &[String]) -> Result<String, AppError> {
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
