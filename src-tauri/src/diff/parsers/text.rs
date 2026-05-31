use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

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
