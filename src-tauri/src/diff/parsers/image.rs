use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

pub struct ImageParser;

impl FileParser for ImageParser {
    fn extract_text(&self, _data: &[u8]) -> Result<String, AppError> {
        Ok(String::new())
    }

    fn detect_type(&self, path: &str, data: &[u8]) -> Option<FileType> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" => {
                let (width, height) = self.get_image_dimensions(data);
                Some(FileType::Image {
                    width,
                    height,
                    format: ext,
                })
            }
            _ => None,
        }
    }
}

impl ImageParser {
    fn get_image_dimensions(&self, _data: &[u8]) -> (u32, u32) {
        // 简化实现：返回 0
        // 实际应该使用 image crate 解析
        (0, 0)
    }
}
