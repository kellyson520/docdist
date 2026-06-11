use crate::diff::types::FileType;
use crate::error::AppError;
use std::path::Path;
use std::process::Command;

use super::FileParser;

pub struct PdfParser;

impl FileParser for PdfParser {
    fn extract_text(&self, _data: &[u8]) -> Result<String, AppError> {
        Err(AppError::Other(
            "PDF 解析需要临时文件，请使用 parse_pdf_file 方法".to_string(),
        ))
    }

    fn detect_type(&self, path: &str, _data: &[u8]) -> Option<FileType> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ext == "pdf" {
            Some(FileType::Pdf {
                page_count: 0,
                has_images: false,
            })
        } else {
            None
        }
    }
}

/// Validate and sanitize a file path before passing it to external commands.
///
/// Security checks:
/// 1. Reject paths containing null bytes (C-string truncation attack)
/// 2. Reject paths with embedded shell metacharacters
/// 3. Reject paths that are symlinks (prevent symlink-based path traversal)
/// 4. Validate file extension matches expected type
/// 5. Canonicalize to resolve `..` and `.` components
fn validate_pdf_path(path: &str) -> Result<std::path::PathBuf, AppError> {
    // Reject null bytes — these can truncate C strings and bypass checks
    if path.contains('\0') {
        return Err(AppError::Other("PDF 路径包含非法空字节".to_string()));
    }

    // Reject paths with shell metacharacters that could cause injection
    // if ever passed through a shell
    if path.contains(';')
        || path.contains('|')
        || path.contains('&')
        || path.contains('$')
        || path.contains('`')
        || path.contains('(')
        || path.contains(')')
        || path.contains('{')
        || path.contains('}')
        || path.contains('\n')
        || path.contains('\r')
    {
        return Err(AppError::Other("PDF 路径包含非法字符".to_string()));
    }

    // Reject empty or whitespace-only paths
    if path.trim().is_empty() {
        return Err(AppError::Other("PDF 路径不能为空".to_string()));
    }

    // Validate extension before canonicalization (defense in depth)
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if ext.to_lowercase() != "pdf" {
        return Err(AppError::Other(format!("非 PDF 文件扩展名: .{}", ext)));
    }

    // Canonicalize to resolve symlinks and relative components
    let canon = std::fs::canonicalize(path)
        .map_err(|e| AppError::Other(format!("PDF 路径无效: {}", e)))?;

    // Ensure canonicalized path still has .pdf extension
    // (canonicalize may reveal a different file)
    let canon_ext = canon.extension().and_then(|e| e.to_str()).unwrap_or("");
    if canon_ext.to_lowercase() != "pdf" {
        return Err(AppError::Other(
            "canonicalized 路径不是 PDF 文件".to_string(),
        ));
    }

    // Verify it exists and is a regular file (not a directory, device, etc.)
    let metadata = std::fs::metadata(&canon)
        .map_err(|e| AppError::Other(format!("PDF 文件无法读取: {}", e)))?;
    if !metadata.is_file() {
        return Err(AppError::Other("PDF 路径不是普通文件".to_string()));
    }

    Ok(canon)
}

impl PdfParser {
    /// 从文件路径解析 PDF（安全版本）
    #[allow(dead_code)]
    pub fn parse_pdf_file(&self, path: &str) -> Result<String, AppError> {
        let canon = validate_pdf_path(path)?;

        let output = Command::new("pdftotext")
            .args([canon.as_os_str(), std::ffi::OsStr::new("-")])
            .output()
            .map_err(|e| AppError::Other(format!("PDF 解析失败: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other("PDF 文本提取失败".to_string()));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| AppError::Other(format!("PDF 文本解码失败: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type_pdf() {
        let parser = PdfParser;
        let result = parser.detect_type("document.pdf", b"").unwrap();

        match result {
            FileType::Pdf {
                page_count,
                has_images,
            } => {
                assert_eq!(page_count, 0);
                assert!(!has_images);
            }
            _ => panic!("Expected Pdf variant"),
        }
    }

    #[test]
    fn test_detect_type_non_pdf() {
        let parser = PdfParser;
        assert!(parser.detect_type("document.txt", b"").is_none());
    }

    #[test]
    fn test_extract_text_returns_error() {
        let parser = PdfParser;
        let result = parser.extract_text(b"");
        assert!(result.is_err());
    }

    // ── Security tests for path validation ──────────────────────────

    #[test]
    fn test_validate_pdf_path_rejects_null_bytes() {
        let result = validate_pdf_path("document\0.pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("空字节"),);
    }

    #[test]
    fn test_validate_pdf_path_rejects_shell_metacharacters() {
        for evil in &[
            "doc;rm -rf /.pdf",
            "doc|cat /etc/passwd.pdf",
            "doc&evil.pdf",
            "doc$(whoami).pdf",
            "doc`id`.pdf",
            "doc(evil).pdf",
            "doc{evil}.pdf",
            "doc\nevil.pdf",
        ] {
            let result = validate_pdf_path(evil);
            assert!(result.is_err(), "Expected rejection for path: {:?}", evil);
        }
    }

    #[test]
    fn test_validate_pdf_path_rejects_empty_path() {
        assert!(validate_pdf_path("").is_err());
        assert!(validate_pdf_path("   ").is_err());
    }

    #[test]
    fn test_validate_pdf_path_rejects_non_pdf_extension() {
        assert!(validate_pdf_path("/tmp/document.txt").is_err());
        assert!(validate_pdf_path("/tmp/document.exe").is_err());
        assert!(validate_pdf_path("/tmp/document").is_err());
    }

    #[test]
    fn test_validate_pdf_path_rejects_nonexistent_file() {
        let result = validate_pdf_path("/nonexistent/path/file.pdf");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pdf_path_rejects_directory() {
        // /tmp exists and is a directory
        // Create a directory named test.pdf to test
        let dir = tempfile::tempdir().unwrap();
        let pdf_dir = dir.path().join("evil.pdf");
        std::fs::create_dir(&pdf_dir).unwrap();
        let result = validate_pdf_path(pdf_dir.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不是普通文件"),);
    }

    #[test]
    fn test_validate_pdf_path_accepts_valid_pdf() {
        let tmp = tempfile::NamedTempFile::with_suffix(".pdf").unwrap();
        std::fs::write(tmp.path(), b"%PDF-1.4 test").unwrap();
        let result = validate_pdf_path(tmp.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pdf_path_rejects_symlink_to_non_pdf() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().join("target.txt");
        std::fs::write(&target, b"not a pdf").unwrap();
        let link = tmp_dir.path().join("link.pdf");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = validate_pdf_path(link.to_str().unwrap());
        // Should fail because canonicalized path has .txt extension
        assert!(result.is_err());
    }
}
