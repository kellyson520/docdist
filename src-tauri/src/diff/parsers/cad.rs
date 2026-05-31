use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

pub struct CadParser;

impl FileParser for CadParser {
    fn extract_text(&self, data: &[u8]) -> Result<String, AppError> {
        String::from_utf8(data.to_vec())
            .map_err(|e| AppError::Other(format!("DXF 解码失败: {}", e)))
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
pub struct CadStructure {
    pub layers: Vec<String>,
    pub entity_count: usize,
}

impl CadParser {
    /// 解析 DXF 结构
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
