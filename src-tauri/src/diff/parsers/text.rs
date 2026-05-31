use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

#[allow(dead_code)]
pub struct TextParser;

impl FileParser for TextParser {
    fn extract_text(&self, data: &[u8]) -> Result<String, AppError> {
        String::from_utf8(data.to_vec())
            .map_err(|e| AppError::Other(format!("文本解码失败: {}", e)))
    }

    fn detect_type(&self, path: &str, _data: &[u8]) -> Option<FileType> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "txt" | "md" | "json" | "xml" | "html" | "css" | "js" | "ts"
            | "jsx" | "tsx" | "py" | "rs" | "go" | "java" | "c" | "cpp"
            | "h" | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" => {
                Some(FileType::Text {
                    encoding: "UTF-8".to_string(),
                    line_ending: "\n".to_string(),
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_utf8() {
        let parser = TextParser;
        let data = "Hello, 世界!".as_bytes();
        let result = parser.extract_text(data).unwrap();
        assert_eq!(result, "Hello, 世界!");
    }

    #[test]
    fn test_extract_text_empty() {
        let parser = TextParser;
        let data = b"";
        let result = parser.extract_text(data).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_detect_type_text_files() {
        let parser = TextParser;

        assert!(parser.detect_type("test.txt", b"").is_some());
        assert!(parser.detect_type("test.md", b"").is_some());
        assert!(parser.detect_type("test.json", b"").is_some());
        assert!(parser.detect_type("test.rs", b"").is_some());
        assert!(parser.detect_type("test.py", b"").is_some());
        assert!(parser.detect_type("test.ts", b"").is_some());
    }

    #[test]
    fn test_detect_type_non_text() {
        let parser = TextParser;

        assert!(parser.detect_type("test.pdf", b"").is_none());
        assert!(parser.detect_type("test.png", b"").is_none());
        assert!(parser.detect_type("test.zip", b"").is_none());
    }

    #[test]
    fn test_detect_type_returns_text_variant() {
        let parser = TextParser;
        let result = parser.detect_type("test.txt", b"").unwrap();

        match result {
            FileType::Text {
                encoding,
                line_ending,
            } => {
                assert_eq!(encoding, "UTF-8");
                assert_eq!(line_ending, "\n");
            }
            _ => panic!("Expected Text variant"),
        }
    }
}
