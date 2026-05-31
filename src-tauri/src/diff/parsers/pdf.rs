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
            Some(FileType::PDF {
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
    pub fn parse_pdf_file(&self, path: &str) -> Result<String, AppError> {
        let output = Command::new("pdftotext")
            .args([path, "-"])
            .output()
            .map_err(|e| AppError::Other(format!("PDF 解析失败: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other("PDF 文本提取失败".to_string()));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| AppError::Other(format!("PDF 文本解码失败: {}", e)))
    }
}
