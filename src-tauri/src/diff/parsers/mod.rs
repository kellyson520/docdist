pub mod cad;
pub mod image;
pub mod pdf;
pub mod text;

use crate::diff::types::FileType;
use crate::error::AppError;

/// 文件解析器 trait
#[allow(dead_code)]
pub trait FileParser: Send + Sync {
    /// 提取文本内容
    fn extract_text(&self, data: &[u8]) -> Result<String, AppError>;

    /// 检测文件类型
    fn detect_type(&self, path: &str, data: &[u8]) -> Option<FileType>;

    /// 是否支持二进制文件
    fn supports_binary(&self) -> bool {
        false
    }
}

/// 解析器注册表
#[allow(dead_code)]
pub struct ParserRegistry {
    parsers: Vec<Box<dyn FileParser>>,
}

#[allow(dead_code)]
impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: Vec::new(),
        };

        // 注册内置解析器
        registry.register(Box::new(text::TextParser));
        // TODO: T3 将注册 PdfParser、CadParser、ImageParser

        registry
    }

    pub fn register(&mut self, parser: Box<dyn FileParser>) {
        self.parsers.push(parser);
    }

    /// 检测文件类型并提取文本
    pub fn detect_and_parse(
        &self,
        path: &str,
        data: &[u8],
    ) -> Result<(FileType, String), AppError> {
        // 先检测类型
        let file_type = self.detect_type(path, data)?;

        // 再提取文本
        let text = self.extract_text(path, data)?;

        Ok((file_type, text))
    }

    fn detect_type(
        &self,
        path: &str,
        data: &[u8],
    ) -> Result<FileType, AppError> {
        for parser in &self.parsers {
            if let Some(file_type) = parser.detect_type(path, data) {
                return Ok(file_type);
            }
        }

        // 默认为二进制
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Ok(FileType::Binary {
            mime_type: mime.to_string(),
            size: data.len() as u64,
        })
    }

    fn extract_text(
        &self,
        path: &str,
        data: &[u8],
    ) -> Result<String, AppError> {
        // Find the correct parser by matching file type detection,
        // then delegate text extraction to that parser.
        for parser in &self.parsers {
            if parser.detect_type(path, data).is_some() {
                return parser.extract_text(data);
            }
        }

        // No parser matched — return empty for unrecognized/binary files
        Ok(String::new())
    }

    fn is_text_file(&self, path: &str) -> bool {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        matches!(
            ext,
            "txt"
                | "md"
                | "json"
                | "xml"
                | "html"
                | "css"
                | "js"
                | "ts"
                | "jsx"
                | "tsx"
                | "py"
                | "rs"
                | "go"
                | "java"
                | "c"
                | "cpp"
                | "h"
                | "hpp"
                | "cs"
                | "rb"
                | "php"
                | "swift"
                | "kt"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ParserRegistry construction ──────────────────────────────────

    #[test]
    fn test_registry_new_has_text_parser() {
        let registry = ParserRegistry::new();
        // TextParser should be registered: a .txt file should be detected as Text
        let result = registry.detect_type("hello.txt", b"");
        assert!(
            result.is_ok(),
            "new() should register TextParser that detects .txt"
        );
        match result.unwrap() {
            FileType::Text { .. } => {} // expected
            other => panic!("expected FileType::Text, got {:?}", other),
        }
    }

    // ── detect_type (private, accessed via registry) ─────────────────

    #[test]
    fn test_registry_detect_type_text_file() {
        let registry = ParserRegistry::new();

        for ext in &[
            "txt", "md", "json", "xml", "html", "css", "js", "ts", "py", "rs",
            "go", "kt",
        ] {
            let path = format!("file.{}", ext);
            let result = registry.detect_type(&path, b"content");
            assert!(result.is_ok(), "{} should be detected", path);
            assert!(
                matches!(result.unwrap(), FileType::Text { .. }),
                "{} should be FileType::Text",
                path
            );
        }
    }

    #[test]
    fn test_registry_detect_type_unknown_falls_back_to_binary() {
        let registry = ParserRegistry::new();
        let data = vec![0u8, 1, 2, 3, 4];

        let result = registry.detect_type("file.unknownext123", &data).unwrap();
        match result {
            FileType::Binary { size, .. } => {
                assert_eq!(size, data.len() as u64);
            }
            other => panic!("expected Binary fallback, got {:?}", other),
        }
    }

    // ── extract_text (private, via registry) ─────────────────────────

    #[test]
    fn test_registry_extract_text_for_known_type() {
        let registry = ParserRegistry::new();
        let content = b"hello world";
        let text = registry.extract_text("readme.md", content).unwrap();
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_registry_extract_text_for_unknown_type() {
        let registry = ParserRegistry::new();
        let text = registry.extract_text("file.bin", b"\x00\x01\x02").unwrap();
        assert_eq!(text, "", "unknown file types should return empty string");
    }

    // ── detect_and_parse (public) ────────────────────────────────────

    #[test]
    fn test_registry_detect_and_parse_text() {
        let registry = ParserRegistry::new();
        let content = "line1\nline2\n".as_bytes();

        let (file_type, text) =
            registry.detect_and_parse("notes.txt", content).unwrap();
        assert!(matches!(file_type, FileType::Text { .. }));
        assert_eq!(text, "line1\nline2\n");
    }

    #[test]
    fn test_registry_detect_and_parse_unknown() {
        let registry = ParserRegistry::new();
        let binary_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic

        let (file_type, text) = registry
            .detect_and_parse("photo.jpg", &binary_data)
            .unwrap();
        assert!(matches!(file_type, FileType::Binary { .. }));
        assert_eq!(text, "", "binary files should yield empty text");
    }

    #[test]
    fn test_registry_detect_and_parse_empty_input() {
        let registry = ParserRegistry::new();
        let (file_type, text) =
            registry.detect_and_parse("empty.rs", b"").unwrap();
        assert!(matches!(file_type, FileType::Text { .. }));
        assert_eq!(text, "");
    }

    // ── is_text_file (private method) ────────────────────────────────

    #[test]
    fn test_is_text_file_known_extensions() {
        let registry = ParserRegistry::new();
        let known = [
            "txt", "md", "json", "xml", "html", "css", "js", "ts", "jsx",
            "tsx", "py", "rs", "go", "java", "c", "cpp", "h", "hpp", "cs",
            "rb", "php", "swift", "kt",
        ];
        for ext in &known {
            let path = format!("src/main.{}", ext);
            assert!(
                registry.is_text_file(&path),
                ".{} should be recognized as text",
                ext
            );
        }
    }

    #[test]
    fn test_is_text_file_unknown_extension() {
        let registry = ParserRegistry::new();
        let unknown =
            ["bin", "exe", "pdf", "png", "zip", "tar", "mp4", "so", "dll"];
        for ext in &unknown {
            let path = format!("file.{}", ext);
            assert!(
                !registry.is_text_file(&path),
                ".{} should NOT be recognized as text",
                ext
            );
        }
    }

    #[test]
    fn test_is_text_file_no_extension() {
        let registry = ParserRegistry::new();
        assert!(!registry.is_text_file("Makefile"));
        assert!(!registry.is_text_file("LICENSE"));
        assert!(!registry.is_text_file("path/to/README"));
    }

    #[test]
    fn test_is_text_file_nested_path() {
        let registry = ParserRegistry::new();
        assert!(registry.is_text_file("src/components/Button.tsx"));
        assert!(registry.is_text_file("/absolute/path/to/file.py"));
        assert!(!registry.is_text_file("deep/nested/dir/file.dat"));
    }
}
