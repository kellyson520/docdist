use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_64;

const CHUNK_SIZE: usize = 4096; // 4KB

pub fn compute_hash(data: &[u8]) -> String {
    format!("{:016x}", xxh3_64(data))
}

pub fn store_file(
    base_dir: &Path,
    file_path: &Path,
) -> Result<(Vec<String>, u64, String), crate::error::AppError> {
    let content = std::fs::read(file_path)?;
    let file_size = content.len() as u64;
    let file_checksum = compute_hash(&content);
    let mut chunk_hashes = Vec::new();

    for chunk in content.chunks(CHUNK_SIZE) {
        let hash = compute_hash(chunk);
        let chunk_dir = base_dir.join(&hash[..2]);
        let chunk_path = chunk_dir.join(&hash);

        if !chunk_path.exists() {
            std::fs::create_dir_all(&chunk_dir)?;
            std::fs::write(&chunk_path, chunk)?;
        }

        chunk_hashes.push(hash);
    }

    Ok((chunk_hashes, file_size, file_checksum))
}

pub fn restore_file(
    base_dir: &Path,
    chunk_hashes: &[String],
    output_path: &Path,
) -> Result<(), crate::error::AppError> {
    let mut content = Vec::new();

    for hash in chunk_hashes {
        let chunk_path = base_dir.join(&hash[..2]).join(hash);
        let chunk_data = std::fs::read(&chunk_path)?;
        content.extend_from_slice(&chunk_data);
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(output_path, &content)?;
    Ok(())
}

pub fn cleanup_chunks(
    base_dir: &Path,
    active_hashes: &[String],
) -> Result<usize, crate::error::AppError> {
    let active_set: std::collections::HashSet<String> =
        active_hashes.iter().cloned().collect();
    let mut removed = 0;

    if !base_dir.exists() {
        return Ok(0);
    }

    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            for chunk_entry in std::fs::read_dir(entry.path())? {
                let chunk_entry = chunk_entry?;
                let name = chunk_entry
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                if !active_set.contains(&name) {
                    std::fs::remove_file(chunk_entry.path())?;
                    removed += 1;
                }
            }
        }
    }

    Ok(removed)
}
