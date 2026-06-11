use crate::diff::types::FileType;
use crate::error::AppError;
use std::process::Command;

use super::FileParser;

/// Allowed pdftotext binary names. Paths must not contain shell metacharacters.
const ALLOWED_PDF_TOOLS: &[&str] = &["pdftotext"];

/// Maximum allowed path length for PDF files to prevent abuse.
const MAX_PDF_PATH_LEN: usize = 4096;

/// Validate that a path contains no shell metacharacters and has a .pdf extension.
fn validate_pdf_path(path: &str) -> Result<std::path::PathBuf, AppError> {
    if path.is_empty() {
        return Err(AppError::Other("PDF 路径不能为空".to_string()));
    }

    if path.len() > MAX_PDF_PATH_LEN {
        return Err(AppError::Other(format!(
            "PDF 路径长度不能超过 {} 字符",
            MAX_PDF_PATH_LEN
        )));
    }

    // Reject shell metacharacters that could be used for injection
    let shell_dangerous =
        ['|', '&', ';', '(', ')', '{', '}', '`', '$', '\n', '\r'];
    if path.chars().any(|c| shell_dangerous.contains(&c)) {
        return Err(AppError::Other("PDF 路径包含非法字符".to_string()));
    }

    let path_obj = std::path::Path::new(path);

    // Reject null bytes
    if path.contains('\0') {
        return Err(AppError::Other("PDF 路径包含空字节".to_string()));
    }

    // Reject path traversal attempts
    let canon = std::fs::canonicalize(path)
        .map_err(|e| AppError::Other(format!("PDF 路径无效: {}", e)))?;

    if !canon.exists() {
        return Err(AppError::Other(format!(
            "PDF 文件不存在: {}",
            canon.display()
        )));
    }

    // Must be a regular file (not device, socket, etc.)
    let metadata = std::fs::metadata(&canon).map_err(|e| {
        AppError::Other(format!("无法读取 PDF 文件元数据: {}", e))
    })?;
    if !metadata.is_file() {
        return Err(AppError::Other("PDF 路径不是普通文件".to_string()));
    }

    // Must have .pdf extension (case-insensitive)
    let ext = path_obj.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !ext.eq_ignore_ascii_case("pdf") {
        return Err(AppError::Other(format!(
            "文件扩展名 '{}' 不是有效的 PDF 扩展名",
            ext
        )));
    }

    Ok(canon)
}

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

impl PdfParser {
    /// 从文件路径解析 PDF
    ///
    /// Security: Path is validated against injection attacks, must have .pdf extension,
    /// must be a regular file, and pdftotext is invoked without shell=true.
    #[allow(dead_code)]
    pub fn parse_pdf_file(&self, path: &str) -> Result<String, AppError> {
        let canon = validate_pdf_path(path)?;

        // Security: invoke pdftotext directly (no shell) with the canonical path
        let tool = ALLOWED_PDF_TOOLS[0];
        let output = Command::new(tool)
            .arg(canon.as_os_str())
            .arg("-")
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
    use tempfile::NamedTempFile;

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
                assert_eq!(has_images, false);
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

    // ── Security: validate_pdf_path tests ─────────────────────────────

    #[test]
    fn test_validate_pdf_path_rejects_empty() {
        let result = validate_pdf_path("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能为空"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_too_long() {
        let long_path = "a".repeat(MAX_PDF_PATH_LEN + 1) + ".pdf";
        let result = validate_pdf_path(&long_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能超过"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_shell_pipe() {
        let result = validate_pdf_path("/tmp/test.pdf|rm -rf /");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_shell_semicolon() {
        let result = validate_pdf_path("/tmp/test.pdf;rm -rf /");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_backtick() {
        let result = validate_pdf_path("/tmp/`whoami`.pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_dollar_sign() {
        let result = validate_pdf_path("/tmp/$(whoami).pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_ampersand() {
        let result = validate_pdf_path("/tmp/test.pdf&evil");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_parens() {
        let result = validate_pdf_path("/tmp/test(1).pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_null_byte() {
        let result = validate_pdf_path("/tmp/test.pdf\0.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("空字节"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_non_pdf_extension() {
        // Create a real file so canonicalize succeeds, but extension check fails
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("exe");
        std::fs::write(&path, b"not a pdf").unwrap();

        let result = validate_pdf_path(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("不是有效的 PDF 扩展名"));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_validate_pdf_path_rejects_nonexistent() {
        let result = validate_pdf_path("/tmp/nonexistent_file_12345.pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无效"));
    }

    #[test]
    fn test_validate_pdf_path_rejects_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let dir_path = tmp.path().join("test.pdf");
        std::fs::create_dir(&dir_path).unwrap();

        let result = validate_pdf_path(dir_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不是普通文件"));
    }

    #[test]
    fn test_validate_pdf_path_accepts_valid_pdf() {
        let tmp = NamedTempFile::with_suffix(".pdf").unwrap();
        std::fs::write(tmp.path(), b"%PDF-1.4 fake content").unwrap();

        let result = validate_pdf_path(tmp.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().to_string_lossy().ends_with(".pdf"));
    }

    #[test]
    fn test_validate_pdf_path_accepts_uppercase_pdf() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let pdf_path = tmp_dir.path().join("test.PDF");
        std::fs::write(&pdf_path, b"%PDF-1.4 fake").unwrap();

        let result = validate_pdf_path(pdf_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pdf_path_rejects_newline_injection() {
        let result = validate_pdf_path("/tmp/test.pdf\nmalicious");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("非法字符"));
    }
}
