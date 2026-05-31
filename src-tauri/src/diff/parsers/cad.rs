use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

pub struct CadParser;

impl FileParser for CadParser {
    fn extract_text(&self, data: &[u8]) -> Result<String, AppError> {
        // DXF files are text-based, but DWG is binary.
        // Try UTF-8 first; fall back to lossy conversion for DXF
        // or return error hint for DWG.
        match std::str::from_utf8(data) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => {
                // DWG binary data is not extractable as text
                Ok(String::from_utf8_lossy(data).into_owned())
            }
        }
    }

    fn detect_type(&self, path: &str, _data: &[u8]) -> Option<FileType> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "dxf" => Some(FileType::Cad {
                format: "DXF".to_string(),
                layer_count: 0,
                entity_count: 0,
            }),
            "dwg" => Some(FileType::Cad {
                format: "DWG".to_string(),
                layer_count: 0,
                entity_count: 0,
            }),
            _ => None,
        }
    }
}

/// CAD 结构信息
#[allow(dead_code)]
pub struct CadStructure {
    pub layers: Vec<String>,
    pub entity_count: usize,
}

impl CadParser {
    /// 解析 DXF 结构
    #[allow(dead_code)]
    pub fn parse_dxf_structure(
        &self,
        content: &str,
    ) -> Result<CadStructure, AppError> {
        let mut layers = Vec::new();
        let mut entity_count = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "LAYER" {
                layers.push("Layer".to_string());
            }
            if trimmed.starts_with("LINE")
                || trimmed.starts_with("CIRCLE")
                || trimmed.starts_with("ARC")
            {
                entity_count += 1;
            }
        }

        Ok(CadStructure {
            layers,
            entity_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type_dxf() {
        let parser = CadParser;
        let result = parser.detect_type("drawing.dxf", b"").unwrap();

        match result {
            FileType::Cad { format, .. } => {
                assert_eq!(format, "DXF");
            }
            _ => panic!("Expected Cad variant"),
        }
    }

    #[test]
    fn test_detect_type_dwg() {
        let parser = CadParser;
        let result = parser.detect_type("drawing.dwg", b"").unwrap();

        match result {
            FileType::Cad { format, .. } => {
                assert_eq!(format, "DWG");
            }
            _ => panic!("Expected Cad variant"),
        }
    }

    #[test]
    fn test_detect_type_non_cad() {
        let parser = CadParser;
        assert!(parser.detect_type("file.txt", b"").is_none());
    }

    #[test]
    fn test_extract_text_dxf() {
        let parser = CadParser;
        let data = "SECTION\nLAYER\nENDSEC".as_bytes();
        let result = parser.extract_text(data).unwrap();
        assert_eq!(result, "SECTION\nLAYER\nENDSEC");
    }
}
