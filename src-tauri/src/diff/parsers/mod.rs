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
        for parser in &self.parsers {
            if parser.supports_binary() || self.is_text_file(path) {
                return parser.extract_text(data);
            }
        }

        // 二进制文件返回空文本
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
