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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type_png() {
        let parser = ImageParser;
        let result = parser.detect_type("image.png", b"").unwrap();

        match result {
            FileType::Image { format, .. } => {
                assert_eq!(format, "png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_detect_type_jpg() {
        let parser = ImageParser;
        let result = parser.detect_type("photo.jpg", b"").unwrap();

        match result {
            FileType::Image { format, .. } => {
                assert_eq!(format, "jpg");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_detect_type_non_image() {
        let parser = ImageParser;
        assert!(parser.detect_type("file.txt", b"").is_none());
    }

    #[test]
    fn test_extract_text_empty() {
        let parser = ImageParser;
        let result = parser.extract_text(b"").unwrap();
        assert_eq!(result, "");
    }
}
