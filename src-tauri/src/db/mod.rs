use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub type DbPool = Pool<SqliteConnectionManager>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub checksum: String,
    pub chunk_count: i64,
    pub note: String,
    pub tags: Vec<String>,
    pub parent_id: Option<String>,
    pub created_at: String,
}

pub fn init_database(
    db_path: &std::path::Path,
) -> Result<DbPool, crate::error::AppError> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA temp_store = MEMORY;",
        )?;
        Ok(())
    });

    let pool = Pool::builder().max_size(5).build(manager)?;

    let conn = pool.get()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS archives (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            checksum TEXT NOT NULL,
            chunk_count INTEGER NOT NULL DEFAULT 0,
            note TEXT DEFAULT '',
            tags TEXT DEFAULT '[]',
            parent_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS chunks (
            hash TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            ref_count INTEGER NOT NULL DEFAULT 1,
            storage_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS archive_chunks (
            archive_id TEXT NOT NULL,
            chunk_hash TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            PRIMARY KEY (archive_id, chunk_index)
        );
        CREATE INDEX IF NOT EXISTS idx_archives_path
            ON archives(file_path);
        CREATE INDEX IF NOT EXISTS idx_archives_parent
            ON archives(parent_id);
        CREATE INDEX IF NOT EXISTS idx_archives_created
            ON archives(created_at DESC);",
    )?;

    Ok(pool)
}

#[allow(dead_code)]
pub fn insert_archive(
    pool: &DbPool,
    archive: &Archive,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    let tags_json = serde_json::to_string(&archive.tags)?;
    conn.execute(
        "INSERT INTO archives (
            id, file_path, file_name, file_size,
            checksum, chunk_count, note, tags,
            parent_id, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
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
    Ok(())
}

#[allow(dead_code)]
pub fn insert_archive_chunks(
    pool: &DbPool,
    archive_id: &str,
    chunks: &[(String, usize)],
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    for (i, (hash, _size)) in chunks.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO archive_chunks
                (archive_id, chunk_hash, chunk_index)
             VALUES (?1, ?2, ?3)",
            params![archive_id, hash, i as i64],
        )?;
    }
    Ok(())
}

fn row_to_archive(row: &rusqlite::Row) -> rusqlite::Result<Archive> {
    let tags_json: String = row.get(7)?;
    Ok(Archive {
        id: row.get(0)?,
        file_path: row.get(1)?,
        file_name: row.get(2)?,
        file_size: row.get(3)?,
        checksum: row.get(4)?,
        chunk_count: row.get(5)?,
        note: row.get(6)?,
        tags: serde_json::from_str(&tags_json).unwrap_or_default(),
        parent_id: row.get(8)?,
        created_at: row.get(9)?,
    })
}

const SELECT_FIELDS: &str = "
    id, file_path, file_name, file_size, checksum,
    chunk_count, note, tags, parent_id, created_at
";

pub fn get_archives(
    pool: &DbPool,
    file_path: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut sql = format!("SELECT {} FROM archives WHERE 1=1", SELECT_FIELDS);
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(path) = file_path {
        if !path.is_empty() {
            sql.push_str(" AND file_path = ?");
            param_values.push(Box::new(path.to_string()));
        }
    }
    if let Some(s) = search {
        if !s.is_empty() {
            sql.push_str(" AND (file_name LIKE ? OR note LIKE ?)");
            let pattern = format!("%{}%", s);
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern));
        }
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 200");

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), row_to_archive)?;

    let archives: Vec<Archive> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read archive row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(archives)
}

pub fn get_archive(
    pool: &DbPool,
    id: &str,
) -> Result<Option<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives WHERE id = ?1",
        SELECT_FIELDS
    ))?;
    let mut rows = stmt.query_map(params![id], row_to_archive)?;
    match rows.next() {
        Some(Ok(archive)) => Ok(Some(archive)),
        _ => Ok(None),
    }
}

#[allow(dead_code)]
pub fn delete_archive(
    pool: &DbPool,
    id: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM archive_chunks WHERE archive_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM archives WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_archive(
    pool: &DbPool,
    id: &str,
    note: &str,
    tags: &[String],
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    let tags_json = serde_json::to_string(tags)?;
    conn.execute(
        "UPDATE archives SET note = ?1, tags = ?2 WHERE id = ?3",
        params![note, tags_json, id],
    )?;
    Ok(())
}

pub fn get_archive_chunks(
    pool: &DbPool,
    archive_id: &str,
) -> Result<Vec<String>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT chunk_hash FROM archive_chunks
         WHERE archive_id = ?1 ORDER BY chunk_index",
    )?;
    let rows =
        stmt.query_map(params![archive_id], |row| row.get::<_, String>(0))?;
    let chunks: Vec<String> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read chunk hash row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(chunks)
}

pub fn get_timeline(
    pool: &DbPool,
    file_path: &str,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives
         WHERE file_path = ?1 ORDER BY created_at DESC",
        SELECT_FIELDS
    ))?;
    let rows = stmt.query_map(params![file_path], row_to_archive)?;
    let archives: Vec<Archive> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read timeline row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(archives)
}

pub fn get_children(
    pool: &DbPool,
    parent_id: &str,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives
         WHERE parent_id = ?1 ORDER BY created_at",
        SELECT_FIELDS
    ))?;
    let rows = stmt.query_map(params![parent_id], row_to_archive)?;
    let archives: Vec<Archive> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read children row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(archives)
}

pub fn get_statistics(
    pool: &DbPool,
) -> Result<serde_json::Value, crate::error::AppError> {
    let conn = pool.get()?;
    let total: i64 =
        conn.query_row("SELECT COUNT(*) FROM archives", [], |r| r.get(0))?;
    let total_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM archives",
        [],
        |r| r.get(0),
    )?;
    let unique_files: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT file_path) FROM archives",
        [],
        |r| r.get(0),
    )?;
    let total_chunks: i64 =
        conn.query_row("SELECT COUNT(*) FROM archive_chunks", [], |r| {
            r.get(0)
        })?;
    Ok(serde_json::json!({
        "total_archives": total,
        "total_size": total_size,
        "unique_files": unique_files,
        "total_chunks": total_chunks,
    }))
}

/// 分页查询存档
pub fn get_archives_paginated(
    pool: &DbPool,
    file_path: Option<&str>,
    search: Option<&str>,
    page: u32,
    page_size: u32,
) -> Result<(Vec<Archive>, i64), crate::error::AppError> {
    let conn = pool.get()?;
    let mut sql = format!("SELECT {} FROM archives WHERE 1=1", SELECT_FIELDS);
    let mut count_sql = "SELECT COUNT(*) FROM archives WHERE 1=1".to_string();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut count_param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
        Vec::new();

    if let Some(path) = file_path {
        if !path.is_empty() {
            sql.push_str(" AND file_path = ?");
            count_sql.push_str(" AND file_path = ?");
            param_values.push(Box::new(path.to_string()));
            count_param_values.push(Box::new(path.to_string()));
        }
    }
    if let Some(s) = search {
        if !s.is_empty() {
            sql.push_str(
                " AND (file_name LIKE ? OR note LIKE ? OR tags LIKE ?)",
            );
            count_sql.push_str(
                " AND (file_name LIKE ? OR note LIKE ? OR tags LIKE ?)",
            );
            let pattern = format!("%{}%", s);
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern.clone()));
            count_param_values.push(Box::new(pattern.clone()));
            count_param_values.push(Box::new(pattern.clone()));
            count_param_values.push(Box::new(pattern));
        }
    }

    // Get total count
    let mut count_stmt = conn.prepare(&count_sql)?;
    let count_params_refs: Vec<&dyn rusqlite::types::ToSql> =
        count_param_values.iter().map(|p| p.as_ref()).collect();
    let total: i64 =
        count_stmt.query_row(count_params_refs.as_slice(), |r| r.get(0))?;

    // Get paginated results — 使用参数绑定而非 format! 拼接
    let offset = (page - 1) * page_size;
    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    // LIMIT/OFFSET 参数追加到 param_values
    param_values.push(Box::new(page_size as i64));
    param_values.push(Box::new(offset as i64));

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), row_to_archive)?;

    let archives: Vec<Archive> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read paginated archive row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((archives, total))
}

/// 批量删除存档
#[allow(dead_code)]
pub fn delete_archives_batch(
    pool: &DbPool,
    ids: &[String],
) -> Result<usize, crate::error::AppError> {
    let conn = pool.get()?;
    let mut deleted = 0;

    for id in ids {
        conn.execute(
            "DELETE FROM archive_chunks WHERE archive_id = ?1",
            params![id],
        )?;
        deleted +=
            conn.execute("DELETE FROM archives WHERE id = ?1", params![id])?;
    }

    Ok(deleted)
}

/// 获取存储统计
pub fn get_storage_stats(
    pool: &DbPool,
) -> Result<serde_json::Value, crate::error::AppError> {
    let conn = pool.get()?;

    let total_chunks: i64 =
        conn.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))?;

    let total_chunk_size: i64 =
        conn.query_row("SELECT COALESCE(SUM(size), 0) FROM chunks", [], |r| {
            r.get(0)
        })?;

    let avg_refs: f64 = conn.query_row(
        "SELECT COALESCE(AVG(ref_count), 0) FROM chunks",
        [],
        |r| r.get(0),
    )?;

    Ok(serde_json::json!({
        "total_chunks": total_chunks,
        "total_chunk_size": total_chunk_size,
        "avg_refs": avg_refs,
    }))
}

/// 插入或更新 chunk（upsert，增加 ref_count）
pub fn upsert_chunk(
    pool: &DbPool,
    hash: &str,
    size: usize,
) -> Result<(), crate::error::AppError> {
    if hash.len() < 2 {
        return Err(crate::error::AppError::Other(format!(
            "chunk hash too short for directory sharding: '{}'",
            hash
        )));
    }
    let conn = pool.get()?;
    let storage_path = format!("{}/{}", &hash[..2], hash);
    conn.execute(
        "INSERT INTO chunks (hash, size, ref_count, storage_path)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(hash) DO UPDATE SET ref_count = ref_count + 1",
        params![hash, size as i64, storage_path],
    )?;
    Ok(())
}

/// 减少 chunk 引用计数，如果归零则标记可清理
pub fn decrement_chunk_ref(
    pool: &DbPool,
    hash: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE chunks SET ref_count = MAX(0, ref_count - 1) WHERE hash = ?1",
        params![hash],
    )?;
    Ok(())
}

/// 删除存档的 chunks 关联记录
#[allow(dead_code)]
pub fn delete_archive_chunks(
    pool: &DbPool,
    archive_id: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM archive_chunks WHERE archive_id = ?1",
        params![archive_id],
    )?;
    Ok(())
}

/// 删除存档记录本身
#[allow(dead_code)]
pub fn delete_archive_record(
    pool: &DbPool,
    id: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM archives WHERE id = ?1", params![id])?;
    Ok(())
}

/// 获取所有活跃的 chunk hash（用于孤儿清理）
pub fn get_all_chunk_hashes(
    pool: &DbPool,
) -> Result<Vec<String>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT DISTINCT chunk_hash FROM archive_chunks")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let hashes: Vec<String> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read chunk hash row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(hashes)
}

/// 获取所有不被引用的 chunks（ref_count = 0）
pub fn get_unreferenced_chunks(
    pool: &DbPool,
) -> Result<Vec<String>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT hash FROM chunks WHERE ref_count <= 0")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let hashes: Vec<String> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read unreferenced chunk row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(hashes)
}

/// 获取指定存档的详细信息（含 chunks）
pub fn get_archive_detail(
    pool: &DbPool,
    id: &str,
) -> Result<Option<(Archive, Vec<String>)>, crate::error::AppError> {
    let archive = get_archive(pool, id)?;
    match archive {
        Some(a) => {
            let chunks = get_archive_chunks(pool, id)?;
            Ok(Some((a, chunks)))
        }
        None => Ok(None),
    }
}
