use crate::diff::types::FileType;
use crate::error::AppError;

use super::FileParser;

#[allow(dead_code)]
pub struct TextParser;

impl FileParser for TextParser {
    fn extract_text(&self, data: &[u8]) -> Result<String, AppError> {
        // Use lossy conversion so non-UTF-8 bytes are replaced with
        // the Unicode replacement character instead of causing a hard error.
        // This handles GBK, Latin-1, BOM-prefixed, and other encodings.
        Ok(String::from_utf8_lossy(data).into_owned())
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

    #[test]
    fn test_extract_text_gbk_bytes_lossy() {
        let parser = TextParser;
        // GBK encoding for "你好" (ni hao) — these byte sequences are invalid UTF-8.
        // from_utf8_lossy should replace them with the Unicode replacement character U+FFFD.
        let gbk_bytes: &[u8] = &[0xC4, 0xE3, 0xBA, 0xC3];
        let result = parser.extract_text(gbk_bytes).unwrap();
        // Each invalid byte sequence gets replaced with the replacement char
        assert!(
            result.contains('\u{FFFD}'),
            "GBK bytes should produce replacement characters"
        );
        // The exact replacement depends on how from_utf8_lossy chunks the invalid bytes,
        // but the output should NOT be empty and should contain replacement chars.
        assert!(!result.is_empty());
    }

    #[test]
    fn test_extract_text_latin1_bytes() {
        let parser = TextParser;
        // Latin-1 bytes for "café": é is 0xE9 in Latin-1, which is invalid as a leading
        // byte in this UTF-8 context (followed by ASCII bytes).
        let latin1_bytes: &[u8] = b"caf";
        // Simple Latin-1 that is also valid UTF-8
        let result = parser.extract_text(latin1_bytes).unwrap();
        assert_eq!(result, "caf");

        // Latin-1-only byte 0xE9 (é) is not valid UTF-8 start byte when followed by non-continuation
        let invalid: &[u8] = &[0xE9, 0x20]; // 0xE9 then space
        let result2 = parser.extract_text(invalid).unwrap();
        assert!(
            result2.contains('\u{FFFD}'),
            "Invalid Latin-1 byte 0xE9 should be replaced"
        );
    }

    #[test]
    fn test_extract_text_with_bom() {
        let parser = TextParser;
        // UTF-8 BOM: 0xEF, 0xBB, 0xBF
        let bom_prefix: &[u8] = &[0xEF, 0xBB, 0xBF];
        let content = "Hello, world!";
        let mut bom_data = bom_prefix.to_vec();
        bom_data.extend_from_slice(content.as_bytes());

        let result = parser.extract_text(&bom_data).unwrap();
        // BOM bytes are valid UTF-8 (U+FEFF), so they remain as a BOM character.
        assert!(
            result.starts_with('\u{FEFF}'),
            "Should preserve BOM character"
        );
        assert!(result.ends_with("Hello, world!"));
        // Total length = BOM char + content
        assert_eq!(result.chars().count(), 1 + content.chars().count());

        // Also test BOM-only input
        let result_bom_only = parser.extract_text(bom_prefix).unwrap();
        assert_eq!(result_bom_only, "\u{FEFF}");
    }

    #[test]
    fn test_detect_type_all_extensions() {
        let parser = TextParser;
        let text_extensions = [
            "txt", "md", "json", "xml", "html", "css", "js", "ts", "jsx",
            "tsx", "py", "rs", "go", "java", "c", "cpp", "h", "hpp", "cs",
            "rb", "php", "swift", "kt",
        ];

        for ext in &text_extensions {
            let path = format!("file.{}", ext);
            let result = parser.detect_type(&path, b"");
            assert!(
                result.is_some(),
                "Extension '{}' should be detected as text, but got None",
                ext
            );

            // Verify it returns FileType::Text with correct encoding/line_ending
            if let Some(FileType::Text {
                encoding,
                line_ending,
            }) = result
            {
                assert_eq!(encoding, "UTF-8", "Encoding mismatch for .{}", ext);
                assert_eq!(
                    line_ending, "\n",
                    "Line ending mismatch for .{}",
                    ext
                );
            } else {
                panic!("Expected FileType::Text for .{}", ext);
            }
        }
    }

    #[test]
    fn test_detect_type_no_extension() {
        let parser = TextParser;
        assert!(parser.detect_type("Makefile", b"").is_none());
        assert!(parser.detect_type("Dockerfile", b"").is_none());
        assert!(parser.detect_type("README", b"").is_none());
        assert!(parser.detect_type("", b"").is_none());
        assert!(parser.detect_type("/some/path/to/file", b"").is_none());
    }

    #[test]
    fn test_detect_type_unknown_extension() {
        let parser = TextParser;
        assert!(parser.detect_type("test.pdf", b"").is_none());
        assert!(parser.detect_type("test.png", b"").is_none());
        assert!(parser.detect_type("test.zip", b"").is_none());
        assert!(parser.detect_type("test.exe", b"").is_none());
        assert!(parser.detect_type("test.mp3", b"").is_none());
        assert!(parser.detect_type("test.avi", b"").is_none());
        assert!(parser.detect_type("test.doc", b"").is_none());
        assert!(parser.detect_type("test.xls", b"").is_none());
        assert!(parser.detect_type("test.bmp", b"").is_none());
        assert!(parser.detect_type("test.unknown", b"").is_none());
    }

    #[test]
    fn test_extract_text_binary_content() {
        let parser = TextParser;
        // Content with null bytes — typical of binary files
        let binary_data: &[u8] = &[0x00, 0x01, 0x02, 0x00, 0xFF, 0xFE, 0x00];
        let result = parser.extract_text(binary_data).unwrap();
        // from_utf8_lossy replaces invalid bytes; 0x00 is valid UTF-8 (NUL),
        // but other high bytes may be replaced.
        assert!(!result.is_empty(), "Should produce non-empty output");
        // 0x00 bytes are preserved as NUL characters
        assert!(result.contains('\0'), "NUL bytes should be preserved");

        // Mostly null bytes
        let null_heavy: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00];
        let result2 = parser.extract_text(null_heavy).unwrap();
        assert_eq!(result2.len(), 5, "NUL bytes should each be preserved");

        // Mix of valid and invalid UTF-8 bytes
        let mixed: &[u8] = &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0xC0, 0xAF];
        let result3 = parser.extract_text(mixed).unwrap();
        assert!(result3.starts_with("Hello"), "Valid ASCII should survive");
        assert!(result3.contains('\0'), "NUL byte should survive");
        assert!(
            result3.contains('\u{FFFD}'),
            "Invalid sequence 0xC0 0xAF should be replaced"
        );
    }
}
