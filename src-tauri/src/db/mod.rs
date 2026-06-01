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
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            CHECK(file_path != ''),
            CHECK(file_name != ''),
            CHECK(checksum != ''),
            CHECK(file_size >= 0),
            CHECK(chunk_count >= 0)
        );
        CREATE TABLE IF NOT EXISTS chunks (
            hash TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            ref_count INTEGER NOT NULL DEFAULT 1,
            storage_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            CHECK(hash != ''),
            CHECK(size >= 0),
            CHECK(ref_count >= 0)
        );
        CREATE TABLE IF NOT EXISTS archive_chunks (
            archive_id TEXT NOT NULL,
            chunk_hash TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            PRIMARY KEY (archive_id, chunk_index),
            CHECK(chunk_index >= 0)
        );
        CREATE TABLE IF NOT EXISTS archive_stars (
            id TEXT PRIMARY KEY,
            archive_id TEXT NOT NULL REFERENCES archives(id) ON DELETE CASCADE,
            label TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(archive_id)
        );
        CREATE INDEX IF NOT EXISTS idx_archive_stars_archive
            ON archive_stars(archive_id);
        CREATE INDEX IF NOT EXISTS idx_archives_path
            ON archives(file_path);
        CREATE INDEX IF NOT EXISTS idx_archives_parent
            ON archives(parent_id);
        CREATE INDEX IF NOT EXISTS idx_archives_created
            ON archives(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_archive_chunks_hash
            ON archive_chunks(chunk_hash);",
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
            sql.push_str(
                " AND (file_name LIKE ? ESCAPE '\\' OR note LIKE ? ESCAPE '\\' OR tags LIKE ? ESCAPE '\\')",
            );
            // 转义 LIKE 通配符，防止用户输入的 % 和 _ 被当作通配符
            let escaped = s
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            param_values.push(Box::new(pattern.clone()));
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

/// 获取所有存档（无 LIMIT，用于树视图等需要完整数据的场景）
pub fn get_all_archives(
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
            sql.push_str(
                " AND (file_name LIKE ? ESCAPE '\\' OR note LIKE ? ESCAPE '\\' OR tags LIKE ? ESCAPE '\\')",
            );
            let escaped = s
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern));
        }
    }
    sql.push_str(" ORDER BY created_at DESC");

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
        Some(Err(e)) => Err(crate::error::AppError::Db(e)),
        None => Ok(None),
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
    let rows = conn.execute(
        "UPDATE archives SET note = ?1, tags = ?2 WHERE id = ?3",
        params![note, tags_json, id],
    )?;
    if rows == 0 {
        return Err(crate::error::AppError::Other("存档不存在".to_string()));
    }
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
    let mut conn = pool.get()?;
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)?;
    let total: i64 =
        tx.query_row("SELECT COUNT(*) FROM archives", [], |r| r.get(0))?;
    let total_size: i64 = tx.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM archives",
        [],
        |r| r.get(0),
    )?;
    let unique_files: i64 = tx.query_row(
        "SELECT COUNT(DISTINCT file_path) FROM archives",
        [],
        |r| r.get(0),
    )?;
    let total_chunks: i64 =
        tx.query_row("SELECT COUNT(*) FROM archive_chunks", [], |r| r.get(0))?;
    tx.commit()?;
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
    // 防御性校验：page 和 page_size 必须 >= 1，否则返回空结果
    if page < 1 || page_size < 1 {
        return Ok((Vec::new(), 0));
    }
    let page_size = page_size.min(500);
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
                " AND (file_name LIKE ? ESCAPE '\\' OR note LIKE ? ESCAPE '\\' OR tags LIKE ? ESCAPE '\\')",
            );
            count_sql.push_str(
                " AND (file_name LIKE ? ESCAPE '\\' OR note LIKE ? ESCAPE '\\' OR tags LIKE ? ESCAPE '\\')",
            );
            // 转义 LIKE 通配符，防止用户输入的 % 和 _ 被当作通配符
            let escaped = s
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
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
    // 使用 u64 避免 (page - 1) * page_size 在 u32 范围内溢出
    let offset: u64 = (page as u64 - 1) * page_size as u64;
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn get_archive_detail(
    pool: &DbPool,
    id: &str,
) -> Result<Option<(Archive, Vec<String>)>, crate::error::AppError> {
    let mut conn = pool.get()?;
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)?;
    let mut archive_stmt = tx.prepare(&format!(
        "SELECT {} FROM archives WHERE id = ?1",
        SELECT_FIELDS
    ))?;
    let mut archive_rows =
        archive_stmt.query_map(rusqlite::params![id], row_to_archive)?;
    let archive = match archive_rows.next() {
        Some(Ok(a)) => a,
        Some(Err(e)) => return Err(crate::error::AppError::Db(e)),
        None => return Ok(None),
    };

    let mut chunk_stmt = tx.prepare(
        "SELECT chunk_hash FROM archive_chunks
         WHERE archive_id = ?1 ORDER BY chunk_index",
    )?;
    let chunk_rows = chunk_stmt
        .query_map(rusqlite::params![id], |row| row.get::<_, String>(0))?;
    let chunks: Vec<String> = chunk_rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read chunk hash row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some((archive, chunks)))
}
/// 标记一个存档为重要版本
#[allow(dead_code)]
pub fn star_archive(
    pool: &DbPool,
    archive_id: &str,
    label: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT OR REPLACE INTO archive_stars (id, archive_id, label) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, archive_id, label],
    )?;
    Ok(())
}

/// 取消标记
#[allow(dead_code)]
pub fn unstar_archive(
    pool: &DbPool,
    archive_id: &str,
) -> Result<(), crate::error::AppError> {
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM archive_stars WHERE archive_id = ?1",
        rusqlite::params![archive_id],
    )?;
    Ok(())
}

/// 获取所有标记的版本
#[allow(dead_code)]
pub fn get_starred_archives(
    pool: &DbPool,
) -> Result<Vec<(Archive, String, String)>, crate::error::AppError> {
    // Returns (Archive, star_id, label)
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {}, s.id as star_id, s.label FROM archives a
         INNER JOIN archive_stars s ON a.id = s.archive_id
         ORDER BY s.created_at DESC",
        SELECT_FIELDS
            .replace("id,", "a.id,")
            .replace("file_path", "a.file_path")
            .replace("file_name", "a.file_name")
            .replace("file_size", "a.file_size")
            .replace("checksum", "a.checksum")
            .replace("chunk_count", "a.chunk_count")
            .replace("note", "a.note")
            .replace("tags", "a.tags")
            .replace("parent_id", "a.parent_id")
            .replace("created_at", "a.created_at")
    ))?;
    let rows = stmt.query_map([], |row| {
        let archive = row_to_archive(row)?;
        let star_id: String = row.get(10)?;
        let label: String = row.get(11)?;
        Ok((archive, star_id, label))
    })?;
    let results: Vec<(Archive, String, String)> = rows
        .map(|r| {
            r.map_err(|e| {
                tracing::warn!("Failed to read starred archive row: {}", e);
                crate::error::AppError::Db(e)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(results)
}

/// 按路径模式搜索存档（支持前缀匹配）
#[allow(dead_code)]
pub fn get_archives_by_path_pattern(
    pool: &DbPool,
    pattern: &str,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives WHERE file_path LIKE ?1 ESCAPE '\\'
         ORDER BY created_at DESC LIMIT 500",
        SELECT_FIELDS
    ))?;
    let escaped = pattern
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let like_pattern = format!("%{}%", escaped);
    let rows =
        stmt.query_map(rusqlite::params![like_pattern], row_to_archive)?;
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

/// 检查存档是否已被标记
#[allow(dead_code)]
pub fn is_archived_starred(
    pool: &DbPool,
    archive_id: &str,
) -> Result<bool, crate::error::AppError> {
    let conn = pool.get()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM archive_stars WHERE archive_id = ?1",
        rusqlite::params![archive_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// 按精确文件路径查询存档
#[allow(dead_code)]
pub fn get_archives_by_file_path(
    pool: &DbPool,
    file_path: &str,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives WHERE file_path = ?1 ORDER BY created_at DESC",
        SELECT_FIELDS
    ))?;
    let rows = stmt.query_map(rusqlite::params![file_path], row_to_archive)?;
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

/// 按目录路径和时间点查询存档（目录前缀匹配 + 时间过滤）
#[allow(dead_code)]
pub fn get_archives_by_dir_before(
    pool: &DbPool,
    dir_path: &str,
    before: &str,
) -> Result<Vec<Archive>, crate::error::AppError> {
    let conn = pool.get()?;
    let pattern = format!("{}/%", dir_path);
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM archives WHERE file_path LIKE ?1 AND created_at <= ?2
         ORDER BY file_path, created_at DESC",
        SELECT_FIELDS
    ))?;
    let rows =
        stmt.query_map(rusqlite::params![pattern, before], row_to_archive)?;
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

/// 获取指定存档的 chunk hash 列表
#[allow(dead_code)]
pub fn get_archive_chunk_hashes(
    pool: &DbPool,
    archive_id: &str,
) -> Result<Vec<String>, crate::error::AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT chunk_hash FROM archive_chunks
         WHERE archive_id = ?1 ORDER BY chunk_index",
    )?;
    let rows = stmt.query_map(rusqlite::params![archive_id], |row| {
        row.get::<_, String>(0)
    })?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;

    /// 创建内存数据库池，max_size=1 确保所有请求复用同一个连接
    fn setup_db() -> DbPool {
        let manager = SqliteConnectionManager::memory().with_init(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA temp_store = MEMORY;",
            )?;
            Ok(())
        });
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("创建内存连接池失败");

        let conn = pool.get().expect("获取连接失败");
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
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                CHECK(file_path != ''),
                CHECK(file_name != ''),
                CHECK(checksum != ''),
                CHECK(file_size >= 0),
                CHECK(chunk_count >= 0)
            );
            CREATE TABLE IF NOT EXISTS chunks (
                hash TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                ref_count INTEGER NOT NULL DEFAULT 1,
                storage_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                CHECK(hash != ''),
                CHECK(size >= 0),
                CHECK(ref_count >= 0)
            );
            CREATE TABLE IF NOT EXISTS archive_chunks (
                archive_id TEXT NOT NULL,
                chunk_hash TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                PRIMARY KEY (archive_id, chunk_index),
                CHECK(chunk_index >= 0)
            );
            CREATE TABLE IF NOT EXISTS archive_stars (
                id TEXT PRIMARY KEY,
                archive_id TEXT NOT NULL REFERENCES archives(id) ON DELETE CASCADE,
                label TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(archive_id)
            );
            CREATE INDEX IF NOT EXISTS idx_archive_stars_archive
                ON archive_stars(archive_id);
            CREATE INDEX IF NOT EXISTS idx_archives_path
                ON archives(file_path);
            CREATE INDEX IF NOT EXISTS idx_archives_parent
                ON archives(parent_id);
            CREATE INDEX IF NOT EXISTS idx_archives_created
                ON archives(created_at DESC);",
        )
        .expect("建表失败");

        pool
    }

    /// 构造一个测试用 Archive
    fn make_archive(id: &str, file_path: &str, file_name: &str) -> Archive {
        Archive {
            id: id.to_string(),
            file_path: file_path.to_string(),
            file_name: file_name.to_string(),
            file_size: 1024,
            checksum: format!("checksum_{}", id),
            chunk_count: 3,
            note: String::new(),
            tags: vec![],
            parent_id: None,
            created_at: "2025-01-01 00:00:00".to_string(),
        }
    }

    // ============================================================
    // 1. init_database 创建表结构 — 内存DB
    // ============================================================
    #[test]
    fn test_init_database_creates_tables() {
        let pool = setup_db();
        let conn = pool.get().unwrap();

        // 验证 archives 表存在
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM archives", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // 验证 chunks 表存在
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // 验证 archive_chunks 表存在
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM archive_chunks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // 验证 foreign_keys 已开启
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    // ============================================================
    // 2. insert_archive + get_archive 正确读写
    // ============================================================
    #[test]
    fn test_insert_and_get_archive() {
        let pool = setup_db();
        let archive = make_archive("a1", "/docs/file.pdf", "file.pdf");

        insert_archive(&pool, &archive).unwrap();

        let result = get_archive(&pool, "a1").unwrap();
        assert!(result.is_some());
        let got = result.unwrap();
        assert_eq!(got.id, "a1");
        assert_eq!(got.file_path, "/docs/file.pdf");
        assert_eq!(got.file_name, "file.pdf");
        assert_eq!(got.file_size, 1024);
        assert_eq!(got.checksum, "checksum_a1");
        assert_eq!(got.chunk_count, 3);
    }

    #[test]
    fn test_get_archive_not_found() {
        let pool = setup_db();
        let result = get_archive(&pool, "nonexistent").unwrap();
        assert!(result.is_none());
    }

    // ============================================================
    // 3. insert_archive 重复 id 报错
    // ============================================================
    #[test]
    fn test_insert_archive_duplicate_id_error() {
        let pool = setup_db();
        let archive = make_archive("dup1", "/docs/file.pdf", "file.pdf");

        insert_archive(&pool, &archive).unwrap();
        let err = insert_archive(&pool, &archive).unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("数据库错误"),
            "期望中文数据库错误，实际: {}",
            msg
        );
    }

    // ============================================================
    // 4. insert_archive_chunks + get_archive_chunks 正确关联
    // ============================================================
    #[test]
    fn test_insert_and_get_archive_chunks() {
        let pool = setup_db();
        let archive = make_archive("a2", "/docs/data.csv", "data.csv");
        insert_archive(&pool, &archive).unwrap();

        let chunks = vec![
            ("hash_00aa".to_string(), 512usize),
            ("hash_01bb".to_string(), 512),
            ("hash_02cc".to_string(), 256),
        ];
        insert_archive_chunks(&pool, "a2", &chunks).unwrap();

        let result = get_archive_chunks(&pool, "a2").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "hash_00aa");
        assert_eq!(result[1], "hash_01bb");
        assert_eq!(result[2], "hash_02cc");
    }

    #[test]
    fn test_get_archive_chunks_empty() {
        let pool = setup_db();
        let result = get_archive_chunks(&pool, "nonexistent").unwrap();
        assert!(result.is_empty());
    }

    // ============================================================
    // 5. get_archives 无过滤返回全部
    // ============================================================
    #[test]
    fn test_get_archives_no_filter_returns_all() {
        let pool = setup_db();
        for i in 0..5 {
            let a = make_archive(
                &format!("all_{}", i),
                &format!("/docs/f{}.pdf", i),
                &format!("f{}.pdf", i),
            );
            insert_archive(&pool, &a).unwrap();
        }

        let archives = get_archives(&pool, None, None).unwrap();
        assert_eq!(archives.len(), 5);
    }

    // ============================================================
    // 6. get_archives file_path 过滤
    // ============================================================
    #[test]
    fn test_get_archives_file_path_filter() {
        let pool = setup_db();
        insert_archive(&pool, &make_archive("fp1", "/path/a.txt", "a.txt"))
            .unwrap();
        insert_archive(&pool, &make_archive("fp2", "/path/b.txt", "b.txt"))
            .unwrap();
        insert_archive(&pool, &make_archive("fp3", "/other/c.txt", "c.txt"))
            .unwrap();

        let archives = get_archives(&pool, Some("/path/a.txt"), None).unwrap();
        assert_eq!(archives.len(), 1);
        assert_eq!(archives[0].id, "fp1");
    }

    #[test]
    fn test_get_archives_empty_path_ignored() {
        let pool = setup_db();
        insert_archive(&pool, &make_archive("ep1", "/docs/x.txt", "x.txt"))
            .unwrap();

        let archives = get_archives(&pool, Some(""), None).unwrap();
        assert_eq!(archives.len(), 1);
    }

    // ============================================================
    // 7. get_archives search 关键词搜索（file_name + note）
    // ============================================================
    #[test]
    fn test_get_archives_search_by_file_name() {
        let pool = setup_db();
        let mut a1 = make_archive("s1", "/docs/report.pdf", "report.pdf");
        a1.note = "月度报告".to_string();
        insert_archive(&pool, &a1).unwrap();

        let mut a2 = make_archive("s2", "/docs/data.csv", "data.csv");
        a2.note = "原始数据".to_string();
        insert_archive(&pool, &a2).unwrap();

        let results = get_archives(&pool, None, Some("report")).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn test_get_archives_search_by_note() {
        let pool = setup_db();
        let mut a1 = make_archive("sn1", "/docs/a.pdf", "a.pdf");
        a1.note = "重要文件".to_string();
        insert_archive(&pool, &a1).unwrap();

        let mut a2 = make_archive("sn2", "/docs/b.pdf", "b.pdf");
        a2.note = "普通文件".to_string();
        insert_archive(&pool, &a2).unwrap();

        let results = get_archives(&pool, None, Some("重要")).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "sn1");
    }

    // ============================================================
    // 8. update_archive 更新 note 和 tags
    // ============================================================
    #[test]
    fn test_update_archive_note_and_tags() {
        let pool = setup_db();
        let archive = make_archive("u1", "/docs/file.pdf", "file.pdf");
        insert_archive(&pool, &archive).unwrap();

        let new_tags = vec!["工作".to_string(), "重要".to_string()];
        update_archive(&pool, "u1", "已审核", &new_tags).unwrap();

        let got = get_archive(&pool, "u1").unwrap().unwrap();
        assert_eq!(got.note, "已审核");
        assert_eq!(got.tags, vec!["工作", "重要"]);
    }

    #[test]
    fn test_update_archive_nonexistent() {
        let pool = setup_db();
        // 更新不存在的记录应返回错误
        let result = update_archive(&pool, "ghost", "note", &[]);
        assert!(result.is_err(), "should error for non-existent archive");
    }

    // ============================================================
    // 9. delete_archive 级联删除 archive_chunks
    // ============================================================
    #[test]
    fn test_delete_archive_cascades_chunks() {
        let pool = setup_db();
        let archive = make_archive("d1", "/docs/file.pdf", "file.pdf");
        insert_archive(&pool, &archive).unwrap();

        let chunks = vec![
            ("dc_hash0".to_string(), 256usize),
            ("dc_hash1".to_string(), 256),
        ];
        insert_archive_chunks(&pool, "d1", &chunks).unwrap();

        // 确认 chunks 存在
        assert_eq!(get_archive_chunks(&pool, "d1").unwrap().len(), 2);

        delete_archive(&pool, "d1").unwrap();

        // archive 已删除
        assert!(get_archive(&pool, "d1").unwrap().is_none());
        // archive_chunks 已删除
        assert!(get_archive_chunks(&pool, "d1").unwrap().is_empty());
    }

    // ============================================================
    // 10. get_archives_paginated 分页正确
    // ============================================================
    #[test]
    fn test_get_archives_paginated() {
        let pool = setup_db();
        // 插入 10 条不同 created_at 的记录
        for i in 0..10 {
            let mut a = make_archive(
                &format!("pg_{}", i),
                "/docs/paginated.pdf",
                "paginated.pdf",
            );
            a.created_at = format!("2025-01-{:02} 00:00:00", i + 1);
            insert_archive(&pool, &a).unwrap();
        }

        // 第1页，每页3条
        let (page1, total) =
            get_archives_paginated(&pool, None, None, 1, 3).unwrap();
        assert_eq!(total, 10);
        assert_eq!(page1.len(), 3);
        // created_at DESC → 最新的在前
        assert_eq!(page1[0].created_at, "2025-01-10 00:00:00");

        // 第2页
        let (page2, total2) =
            get_archives_paginated(&pool, None, None, 2, 3).unwrap();
        assert_eq!(total2, 10);
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].created_at, "2025-01-07 00:00:00");

        // 第4页只有1条
        let (page4, total4) =
            get_archives_paginated(&pool, None, None, 4, 3).unwrap();
        assert_eq!(total4, 10);
        assert_eq!(page4.len(), 1);
    }

    #[test]
    fn test_get_archives_paginated_empty() {
        let pool = setup_db();
        let (results, total) =
            get_archives_paginated(&pool, None, None, 1, 10).unwrap();
        assert!(results.is_empty());
        assert_eq!(total, 0);
    }

    // ============================================================
    // 11. get_timeline 按 created_at DESC 排序
    // ============================================================
    #[test]
    fn test_get_timeline_ordered_desc() {
        let pool = setup_db();
        let path = "/docs/timeline_doc.pdf";

        let mut a1 = make_archive("tl1", path, "timeline_doc.pdf");
        a1.created_at = "2025-01-01 10:00:00".to_string();
        insert_archive(&pool, &a1).unwrap();

        let mut a2 = make_archive("tl2", path, "timeline_doc.pdf");
        a2.created_at = "2025-06-15 12:00:00".to_string();
        insert_archive(&pool, &a2).unwrap();

        let mut a3 = make_archive("tl3", path, "timeline_doc.pdf");
        a3.created_at = "2025-03-20 08:00:00".to_string();
        insert_archive(&pool, &a3).unwrap();

        let timeline = get_timeline(&pool, path).unwrap();
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].id, "tl2"); // 2025-06-15 最新
        assert_eq!(timeline[1].id, "tl3"); // 2025-03-20
        assert_eq!(timeline[2].id, "tl1"); // 2025-01-01 最早
    }

    // ============================================================
    // 12. get_children 按 parent_id 查询
    // ============================================================
    #[test]
    fn test_get_children_by_parent_id() {
        let pool = setup_db();
        let mut parent =
            make_archive("parent1", "/docs/parent.pdf", "parent.pdf");
        parent.created_at = "2025-01-01 00:00:00".to_string();
        insert_archive(&pool, &parent).unwrap();

        let mut child1 =
            make_archive("child1", "/docs/child1.pdf", "child1.pdf");
        child1.parent_id = Some("parent1".to_string());
        child1.created_at = "2025-01-02 00:00:00".to_string();
        insert_archive(&pool, &child1).unwrap();

        let mut child2 =
            make_archive("child2", "/docs/child2.pdf", "child2.pdf");
        child2.parent_id = Some("parent1".to_string());
        child2.created_at = "2025-01-03 00:00:00".to_string();
        insert_archive(&pool, &child2).unwrap();

        // 不相关记录
        let mut other = make_archive("other1", "/docs/other.pdf", "other.pdf");
        other.parent_id = Some("someone_else".to_string());
        insert_archive(&pool, &other).unwrap();

        let children = get_children(&pool, "parent1").unwrap();
        assert_eq!(children.len(), 2);
        // get_children 按 created_at ASC 排序
        assert_eq!(children[0].id, "child1");
        assert_eq!(children[1].id, "child2");
    }

    #[test]
    fn test_get_children_empty() {
        let pool = setup_db();
        let children = get_children(&pool, "no_parent").unwrap();
        assert!(children.is_empty());
    }

    // ============================================================
    // 13. get_statistics 统计值正确
    // ============================================================
    #[test]
    fn test_get_statistics() {
        let pool = setup_db();

        let mut a1 = make_archive("stat1", "/docs/a.pdf", "a.pdf");
        a1.file_size = 1000;
        a1.parent_id = None;
        insert_archive(&pool, &a1).unwrap();

        let mut a2 = make_archive("stat2", "/docs/b.pdf", "b.pdf");
        a2.file_size = 2000;
        insert_archive(&pool, &a2).unwrap();

        // 同 file_path 的另一条记录
        let mut a3 = make_archive("stat3", "/docs/a.pdf", "a.pdf");
        a3.file_size = 500;
        insert_archive(&pool, &a3).unwrap();

        // 添加 archive_chunks
        insert_archive_chunks(&pool, "stat1", &[("sc_h1".to_string(), 512)])
            .unwrap();
        insert_archive_chunks(
            &pool,
            "stat2",
            &[("sc_h2".to_string(), 512), ("sc_h3".to_string(), 256)],
        )
        .unwrap();

        let stats = get_statistics(&pool).unwrap();
        assert_eq!(stats["total_archives"], 3);
        assert_eq!(stats["total_size"], 3500); // 1000+2000+500
        assert_eq!(stats["unique_files"], 2); // /docs/a.pdf, /docs/b.pdf
        assert_eq!(stats["total_chunks"], 3);
    }

    #[test]
    fn test_get_statistics_empty() {
        let pool = setup_db();
        let stats = get_statistics(&pool).unwrap();
        assert_eq!(stats["total_archives"], 0);
        assert_eq!(stats["total_size"], 0);
        assert_eq!(stats["unique_files"], 0);
        assert_eq!(stats["total_chunks"], 0);
    }

    // ============================================================
    // 14. upsert_chunk ref_count 管理
    // ============================================================
    #[test]
    fn test_upsert_chunk_new() {
        let pool = setup_db();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();

        let conn = pool.get().unwrap();
        let (ref_count, size): (i64, i64) = conn
            .query_row(
                "SELECT ref_count, size FROM chunks WHERE hash = ?1",
                params!["abcdef1234567890"],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(ref_count, 1);
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_upsert_chunk_increments_ref_count() {
        let pool = setup_db();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();

        let conn = pool.get().unwrap();
        let ref_count: i64 = conn
            .query_row(
                "SELECT ref_count FROM chunks WHERE hash = ?1",
                params!["abcdef1234567890"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ref_count, 3);
    }

    #[test]
    fn test_upsert_chunk_short_hash_error() {
        let pool = setup_db();
        let err = upsert_chunk(&pool, "a", 100).unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("chunk hash too short"),
            "期望 hash too short 错误，实际: {}",
            msg
        );
    }

    #[test]
    fn test_upsert_chunk_storage_path() {
        let pool = setup_db();
        let hash = "ab12cd34ef567890";
        upsert_chunk(&pool, hash, 512).unwrap();

        let conn = pool.get().unwrap();
        let storage_path: String = conn
            .query_row(
                "SELECT storage_path FROM chunks WHERE hash = ?1",
                params![hash],
                |r| r.get(0),
            )
            .unwrap();
        // storage_path 格式: 前两位/hash
        assert_eq!(storage_path, "ab/ab12cd34ef567890");
    }

    // ============================================================
    // 15. decrement_chunk_ref 正确递减
    // ============================================================
    #[test]
    fn test_decrement_chunk_ref() {
        let pool = setup_db();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        // ref_count = 3

        decrement_chunk_ref(&pool, "abcdef1234567890").unwrap();

        let conn = pool.get().unwrap();
        let ref_count: i64 = conn
            .query_row(
                "SELECT ref_count FROM chunks WHERE hash = ?1",
                params!["abcdef1234567890"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ref_count, 2);
    }

    #[test]
    fn test_decrement_chunk_ref_clamps_at_zero() {
        let pool = setup_db();
        upsert_chunk(&pool, "abcdef1234567890", 1024).unwrap();
        // ref_count = 1

        decrement_chunk_ref(&pool, "abcdef1234567890").unwrap();
        decrement_chunk_ref(&pool, "abcdef1234567890").unwrap(); // 再减一次

        let conn = pool.get().unwrap();
        let ref_count: i64 = conn
            .query_row(
                "SELECT ref_count FROM chunks WHERE hash = ?1",
                params!["abcdef1234567890"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ref_count, 0, "ref_count 应该被 MAX(0, ...) 钳制为 0");
    }

    #[test]
    fn test_decrement_chunk_ref_nonexistent() {
        let pool = setup_db();
        // 不存在的 hash 不会报错，只是 0 行受影响
        decrement_chunk_ref(&pool, "nonexistent123456").unwrap();
    }

    // ============================================================
    // 16. tags JSON 序列化/反序列化往返
    // ============================================================
    #[test]
    fn test_tags_json_roundtrip() {
        let pool = setup_db();
        let mut archive = make_archive("tag1", "/docs/tag.pdf", "tag.pdf");
        archive.tags = vec![
            "标签A".to_string(),
            "标签B".to_string(),
            "special:chars/here".to_string(),
        ];
        insert_archive(&pool, &archive).unwrap();

        let got = get_archive(&pool, "tag1").unwrap().unwrap();
        assert_eq!(got.tags.len(), 3);
        assert_eq!(got.tags[0], "标签A");
        assert_eq!(got.tags[1], "标签B");
        assert_eq!(got.tags[2], "special:chars/here");
    }

    #[test]
    fn test_tags_json_empty_vec() {
        let pool = setup_db();
        let archive = make_archive("tag_e", "/docs/e.pdf", "e.pdf");
        insert_archive(&pool, &archive).unwrap();

        let got = get_archive(&pool, "tag_e").unwrap().unwrap();
        assert!(got.tags.is_empty());
    }

    #[test]
    fn test_tags_json_after_update() {
        let pool = setup_db();
        let archive = make_archive("tag_u", "/docs/u.pdf", "u.pdf");
        insert_archive(&pool, &archive).unwrap();

        let tags = vec!["新标签1".to_string(), "新标签2".to_string()];
        update_archive(&pool, "tag_u", "更新后的备注", &tags).unwrap();

        let got = get_archive(&pool, "tag_u").unwrap().unwrap();
        assert_eq!(got.tags, vec!["新标签1", "新标签2"]);
        assert_eq!(got.note, "更新后的备注");
    }

    // ============================================================
    // 额外覆盖: delete_archive_record, delete_archive_chunks,
    // get_archive_detail, get_all_chunk_hashes, get_unreferenced_chunks,
    // delete_archives_batch, get_storage_stats
    // ============================================================
    #[test]
    fn test_delete_archive_chunks() {
        let pool = setup_db();
        let archive = make_archive("dac1", "/docs/x.pdf", "x.pdf");
        insert_archive(&pool, &archive).unwrap();
        insert_archive_chunks(
            &pool,
            "dac1",
            &[("h0".to_string(), 64), ("h1".to_string(), 64)],
        )
        .unwrap();

        delete_archive_chunks(&pool, "dac1").unwrap();
        assert!(get_archive_chunks(&pool, "dac1").unwrap().is_empty());
        // archive 本身还在
        assert!(get_archive(&pool, "dac1").unwrap().is_some());
    }

    #[test]
    fn test_delete_archive_record() {
        let pool = setup_db();
        let archive = make_archive("dar1", "/docs/y.pdf", "y.pdf");
        insert_archive(&pool, &archive).unwrap();

        delete_archive_record(&pool, "dar1").unwrap();
        assert!(get_archive(&pool, "dar1").unwrap().is_none());
    }

    #[test]
    fn test_get_archive_detail() {
        let pool = setup_db();
        let archive = make_archive("det1", "/docs/z.pdf", "z.pdf");
        insert_archive(&pool, &archive).unwrap();
        insert_archive_chunks(
            &pool,
            "det1",
            &[("dh0".to_string(), 128), ("dh1".to_string(), 256)],
        )
        .unwrap();

        let detail = get_archive_detail(&pool, "det1").unwrap();
        assert!(detail.is_some());
        let (a, chunks) = detail.unwrap();
        assert_eq!(a.id, "det1");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "dh0");
        assert_eq!(chunks[1], "dh1");
    }

    #[test]
    fn test_get_archive_detail_not_found() {
        let pool = setup_db();
        let detail = get_archive_detail(&pool, "nope").unwrap();
        assert!(detail.is_none());
    }

    #[test]
    fn test_get_all_chunk_hashes() {
        let pool = setup_db();
        let a1 = make_archive("gach1", "/docs/q.pdf", "q.pdf");
        insert_archive(&pool, &a1).unwrap();
        insert_archive_chunks(
            &pool,
            "gach1",
            &[("ha0".to_string(), 64), ("ha1".to_string(), 64)],
        )
        .unwrap();

        let a2 = make_archive("gach2", "/docs/r.pdf", "r.pdf");
        insert_archive(&pool, &a2).unwrap();
        // 共享 ha1
        insert_archive_chunks(
            &pool,
            "gach2",
            &[("ha1".to_string(), 64), ("ha2".to_string(), 64)],
        )
        .unwrap();

        let hashes = get_all_chunk_hashes(&pool).unwrap();
        assert_eq!(hashes.len(), 3); // ha0, ha1, ha2 (DISTINCT)
    }

    #[test]
    fn test_get_unreferenced_chunks() {
        let pool = setup_db();
        // ref_count = 1 → upsert once
        upsert_chunk(&pool, "unref_abcd12345678", 100).unwrap();
        // ref_count 减到 0
        decrement_chunk_ref(&pool, "unref_abcd12345678").unwrap();

        let unreferenced = get_unreferenced_chunks(&pool).unwrap();
        assert_eq!(unreferenced.len(), 1);
        assert_eq!(unreferenced[0], "unref_abcd12345678");
    }

    #[test]
    fn test_delete_archives_batch() {
        let pool = setup_db();
        for i in 0..3 {
            let a = make_archive(
                &format!("batch_{}", i),
                "/docs/batch.pdf",
                "batch.pdf",
            );
            insert_archive(&pool, &a).unwrap();
            insert_archive_chunks(
                &pool,
                &format!("batch_{}", i),
                &[("bh0".to_string(), 64)],
            )
            .unwrap();
        }

        let ids = vec![
            "batch_0".to_string(),
            "batch_1".to_string(),
            "batch_2".to_string(),
        ];
        let deleted = delete_archives_batch(&pool, &ids).unwrap();
        assert_eq!(deleted, 3);

        // 所有 chunks 也应被删除
        for id in &ids {
            assert!(get_archive_chunks(&pool, id).unwrap().is_empty());
            assert!(get_archive(&pool, id).unwrap().is_none());
        }
    }

    #[test]
    fn test_get_storage_stats() {
        let pool = setup_db();
        upsert_chunk(&pool, "ss_aabbccddee112233", 500).unwrap();
        upsert_chunk(&pool, "ss_aabbccddee112233", 500).unwrap(); // ref_count=2
        upsert_chunk(&pool, "ss_1122334455667788", 300).unwrap();

        let stats = get_storage_stats(&pool).unwrap();
        assert_eq!(stats["total_chunks"], 2);
        assert_eq!(stats["total_chunk_size"], 800); // 500+300
                                                    // avg_refs = (2+1)/2 = 1.5
        let avg = stats["avg_refs"].as_f64().unwrap();
        assert!(
            (avg - 1.5).abs() < 0.01,
            "avg_refs 应约为 1.5，实际: {}",
            avg
        );
    }

    #[test]
    fn test_get_storage_stats_empty() {
        let pool = setup_db();
        let stats = get_storage_stats(&pool).unwrap();
        assert_eq!(stats["total_chunks"], 0);
        assert_eq!(stats["total_chunk_size"], 0);
        assert_eq!(stats["avg_refs"], 0.0);
    }

    // ============================================================
    // CHECK 约束验证测试
    // ============================================================

    /// 辅助函数：用原始 SQL 直接插入 archive，绕过 Rust 代码中的验证
    fn raw_insert_archive(
        conn: &rusqlite::Connection,
        id: &str,
        file_path: &str,
        file_name: &str,
        file_size: i64,
        checksum: &str,
        chunk_count: i64,
    ) -> Result<usize, rusqlite::Error> {
        conn.execute(
            "INSERT INTO archives (id, file_path, file_name, file_size, checksum, chunk_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, file_path, file_name, file_size, checksum, chunk_count],
        )
    }

    /// 辅助函数：用原始 SQL 直接插入 chunk
    fn raw_insert_chunk(
        conn: &rusqlite::Connection,
        hash: &str,
        size: i64,
        ref_count: i64,
        storage_path: &str,
    ) -> Result<usize, rusqlite::Error> {
        conn.execute(
            "INSERT INTO chunks (hash, size, ref_count, storage_path)
             VALUES (?1, ?2, ?3, ?4)",
            params![hash, size, ref_count, storage_path],
        )
    }

    /// 辅助函数：用原始 SQL 直接插入 archive_chunk
    fn raw_insert_archive_chunk(
        conn: &rusqlite::Connection,
        archive_id: &str,
        chunk_hash: &str,
        chunk_index: i64,
    ) -> Result<usize, rusqlite::Error> {
        conn.execute(
            "INSERT INTO archive_chunks (archive_id, chunk_hash, chunk_index)
             VALUES (?1, ?2, ?3)",
            params![archive_id, chunk_hash, chunk_index],
        )
    }

    // --- archives 表 CHECK 约束 ---

    #[test]
    fn test_check_archives_file_path_not_empty() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err =
            raw_insert_archive(&conn, "chk1", "", "file.pdf", 100, "hash1", 0)
                .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败，实际: {}",
            err
        );
    }

    #[test]
    fn test_check_archives_file_name_not_empty() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err = raw_insert_archive(
            &conn,
            "chk2",
            "/path/f.pdf",
            "",
            100,
            "hash2",
            0,
        )
        .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败，实际: {}",
            err
        );
    }

    #[test]
    fn test_check_archives_checksum_not_empty() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err = raw_insert_archive(
            &conn,
            "chk3",
            "/path/f.pdf",
            "f.pdf",
            100,
            "",
            0,
        )
        .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败，实际: {}",
            err
        );
    }

    #[test]
    fn test_check_archives_file_size_non_negative() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err = raw_insert_archive(
            &conn,
            "chk4",
            "/path/f.pdf",
            "f.pdf",
            -1,
            "hash4",
            0,
        )
        .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（file_size < 0），实际: {}",
            err
        );
    }

    #[test]
    fn test_check_archives_file_size_zero_allowed() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        // file_size=0 应该成功
        raw_insert_archive(
            &conn,
            "chk4b",
            "/path/f.pdf",
            "f.pdf",
            0,
            "hash4b",
            0,
        )
        .unwrap();
    }

    #[test]
    fn test_check_archives_chunk_count_non_negative() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err = raw_insert_archive(
            &conn,
            "chk5",
            "/path/f.pdf",
            "f.pdf",
            100,
            "hash5",
            -1,
        )
        .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（chunk_count < 0），实际: {}",
            err
        );
    }

    // --- chunks 表 CHECK 约束 ---

    #[test]
    fn test_check_chunks_hash_not_empty() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err = raw_insert_chunk(&conn, "", 100, 1, "ab/ab1234").unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（hash 为空），实际: {}",
            err
        );
    }

    #[test]
    fn test_check_chunks_size_non_negative() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err =
            raw_insert_chunk(&conn, "ab12cd345678", -1, 1, "ab/ab12cd345678")
                .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（size < 0），实际: {}",
            err
        );
    }

    #[test]
    fn test_check_chunks_size_zero_allowed() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        // size=0 应该成功
        raw_insert_chunk(&conn, "ab12cd345678", 0, 1, "ab/ab12cd345678")
            .unwrap();
    }

    #[test]
    fn test_check_chunks_ref_count_non_negative() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let err =
            raw_insert_chunk(&conn, "ab12cd345678", 100, -1, "ab/ab12cd345678")
                .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（ref_count < 0），实际: {}",
            err
        );
    }

    // --- archive_chunks 表 CHECK 约束 ---

    #[test]
    fn test_check_archive_chunks_index_non_negative() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        // 先创建一个合法的 archive
        raw_insert_archive(
            &conn,
            "ac1",
            "/path/f.pdf",
            "f.pdf",
            100,
            "hash1",
            0,
        )
        .unwrap();

        let err = raw_insert_archive_chunk(&conn, "ac1", "chunk_hash1", -1)
            .unwrap_err();
        assert!(
            format!("{}", err).contains("CHECK"),
            "期望 CHECK 约束失败（chunk_index < 0），实际: {}",
            err
        );
    }

    #[test]
    fn test_check_archive_chunks_index_zero_allowed() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        raw_insert_archive(
            &conn,
            "ac2",
            "/path/f.pdf",
            "f.pdf",
            100,
            "hash2",
            0,
        )
        .unwrap();

        // chunk_index=0 应该成功
        raw_insert_archive_chunk(&conn, "ac2", "chunk_hash2", 0).unwrap();
    }

    // --- 正常数据不受 CHECK 约束影响 ---

    #[test]
    fn test_check_constraints_valid_data_passes() {
        let pool = setup_db();
        let archive = make_archive("valid1", "/docs/valid.pdf", "valid.pdf");
        insert_archive(&pool, &archive).unwrap();

        {
            let conn = pool.get().unwrap();
            raw_insert_chunk(
                &conn,
                "abcdef1234567890",
                1024,
                2,
                "ab/abcdef1234567890",
            )
            .unwrap();
            raw_insert_archive_chunk(&conn, "valid1", "abcdef1234567890", 0)
                .unwrap();
        }

        // 验证全部成功读回
        let got = get_archive(&pool, "valid1").unwrap();
        assert!(got.is_some());
    }

    // ============================================================
    // archive_stars 表功能测试
    // ============================================================

    #[test]
    fn test_init_database_creates_archive_stars_table() {
        let pool = setup_db();
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM archive_stars", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_star_archive_and_is_archived_starred() {
        let pool = setup_db();
        let archive = make_archive("star1", "/docs/v1.pdf", "v1.pdf");
        insert_archive(&pool, &archive).unwrap();

        // Initially not starred
        assert!(!is_archived_starred(&pool, "star1").unwrap());

        // Star it
        star_archive(&pool, "star1", "重要版本").unwrap();

        // Now starred
        assert!(is_archived_starred(&pool, "star1").unwrap());
    }

    #[test]
    fn test_star_archive_replaces_label() {
        let pool = setup_db();
        let archive = make_archive("star_r", "/docs/r.pdf", "r.pdf");
        insert_archive(&pool, &archive).unwrap();

        star_archive(&pool, "star_r", "旧标签").unwrap();
        star_archive(&pool, "star_r", "新标签").unwrap();

        let starred = get_starred_archives(&pool).unwrap();
        assert_eq!(starred.len(), 1);
        assert_eq!(starred[0].2, "新标签");
    }

    #[test]
    fn test_unstar_archive() {
        let pool = setup_db();
        let archive = make_archive("star2", "/docs/v2.pdf", "v2.pdf");
        insert_archive(&pool, &archive).unwrap();

        star_archive(&pool, "star2", "标签").unwrap();
        assert!(is_archived_starred(&pool, "star2").unwrap());

        unstar_archive(&pool, "star2").unwrap();
        assert!(!is_archived_starred(&pool, "star2").unwrap());
    }

    #[test]
    fn test_unstar_archive_nonexistent() {
        let pool = setup_db();
        // 取消标记不存在的记录不报错，只是 0 行受影响
        unstar_archive(&pool, "nonexistent").unwrap();
    }

    #[test]
    fn test_get_starred_archives_empty() {
        let pool = setup_db();
        let starred = get_starred_archives(&pool).unwrap();
        assert!(starred.is_empty());
    }

    #[test]
    fn test_get_starred_archives_returns_data() {
        let pool = setup_db();
        let a1 = make_archive("gs1", "/docs/a.pdf", "a.pdf");
        let a2 = make_archive("gs2", "/docs/b.pdf", "b.pdf");
        insert_archive(&pool, &a1).unwrap();
        insert_archive(&pool, &a2).unwrap();

        star_archive(&pool, "gs1", "版本一").unwrap();
        star_archive(&pool, "gs2", "版本二").unwrap();

        let starred = get_starred_archives(&pool).unwrap();
        assert_eq!(starred.len(), 2);
        // 按 created_at DESC 排序，后插入的在前
        let labels: Vec<&str> =
            starred.iter().map(|(_, _, l)| l.as_str()).collect();
        assert!(labels.contains(&"版本一"));
        assert!(labels.contains(&"版本二"));
    }

    #[test]
    fn test_is_archived_starred_nonexistent() {
        let pool = setup_db();
        assert!(!is_archived_starred(&pool, "no_such_id").unwrap());
    }

    #[test]
    fn test_get_archives_by_path_pattern() {
        let pool = setup_db();
        insert_archive(
            &pool,
            &make_archive("pp1", "/docs/reports/2025/q1.pdf", "q1.pdf"),
        )
        .unwrap();
        insert_archive(
            &pool,
            &make_archive("pp2", "/docs/reports/2025/q2.pdf", "q2.pdf"),
        )
        .unwrap();
        insert_archive(
            &pool,
            &make_archive("pp3", "/images/photo.jpg", "photo.jpg"),
        )
        .unwrap();

        let results =
            get_archives_by_path_pattern(&pool, "reports/2025").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|a| a.file_path.contains("reports/2025")));
    }

    #[test]
    fn test_get_archives_by_path_pattern_no_match() {
        let pool = setup_db();
        insert_archive(&pool, &make_archive("pp_nm", "/docs/a.pdf", "a.pdf"))
            .unwrap();

        let results =
            get_archives_by_path_pattern(&pool, "nonexistent_path").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_archives_by_path_pattern_escapes_wildcards() {
        let pool = setup_db();
        // 文件路径中包含 % 和 _ 等 LIKE 通配符
        insert_archive(
            &pool,
            &make_archive("pp_esc", "/docs/100%_done/report.pdf", "report.pdf"),
        )
        .unwrap();
        insert_archive(
            &pool,
            &make_archive("pp_other", "/docs/other/report.pdf", "report.pdf"),
        )
        .unwrap();

        let results = get_archives_by_path_pattern(&pool, "100%_done").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pp_esc");
    }

    #[test]
    fn test_star_archive_cascade_delete() {
        let pool = setup_db();
        let archive = make_archive("star_cd", "/docs/cd.pdf", "cd.pdf");
        insert_archive(&pool, &archive).unwrap();
        star_archive(&pool, "star_cd", "级联测试").unwrap();

        assert!(is_archived_starred(&pool, "star_cd").unwrap());

        // 删除 archive 后，star 记录应因 CASCADE 被自动删除
        delete_archive(&pool, "star_cd").unwrap();
        assert!(!is_archived_starred(&pool, "star_cd").unwrap());
    }
}
