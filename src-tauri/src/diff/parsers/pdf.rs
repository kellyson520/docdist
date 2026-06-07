use crate::diff::types::FileType;
use crate::error::AppError;
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

impl PdfParser {
    /// 从文件路径解析 PDF
    #[allow(dead_code)]
    pub fn parse_pdf_file(&self, path: &str) -> Result<String, AppError> {
        let canon = std::fs::canonicalize(path)
            .map_err(|e| AppError::Other(format!("PDF 路径无效: {}", e)))?;

        if !canon.exists() {
            return Err(AppError::Other(format!(
                "PDF 文件不存在: {}",
                canon.display()
            )));
        }

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
}
