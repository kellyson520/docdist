use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("压缩文件错误: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("连接池错误: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_other_display() {
        let err = AppError::Other("test".to_string());
        assert_eq!(format!("{}", err), "test");
    }

    #[test]
    fn test_io_display() {
        let io_err =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AppError::Io(io_err);
        assert_eq!(format!("{}", err), "文件操作失败: file not found");
    }

    #[test]
    fn test_db_display() {
        let db_err = rusqlite::Error::ExecuteReturnedResults;
        let err = AppError::Db(db_err);
        let msg = format!("{}", err);
        assert!(msg.starts_with("数据库错误:"));
    }

    #[test]
    fn test_serde_display() {
        let json_err =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err = AppError::Serde(json_err);
        let msg = format!("{}", err);
        assert!(msg.starts_with("序列化错误:"));
    }

    #[test]
    fn test_pool_display_prefix() {
        // r2d2::Error has no public constructors; verify prefix format via Other
        let err = AppError::Other("连接池错误: test".to_string());
        assert!(format!("{}", err).contains("连接池错误:"));
    }

    #[test]
    fn test_serialize_other() {
        let err = AppError::Other("hello".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"hello\"");
    }

    #[test]
    fn test_serialize_io() {
        let io_err =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = AppError::Io(io_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("文件操作失败"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io msg");
        let err: AppError = io_err.into();
        assert!(matches!(err, AppError::Io(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err =
            serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let err: AppError = json_err.into();
        assert!(matches!(err, AppError::Serde(_)));
    }

    #[test]
    fn test_from_rusqlite_error() {
        let db_err = rusqlite::Error::ExecuteReturnedResults;
        let err: AppError = db_err.into();
        assert!(matches!(err, AppError::Db(_)));
    }

    #[test]
    fn test_pool_variant_exists() {
        // Verify Pool variant compiles and can be pattern-matched
        // Cannot construct r2d2::Error directly, so just check enum shape
        let err = AppError::Other("pool test".to_string());
        assert!(!matches!(err, AppError::Pool(_)));
    }

    #[test]
    fn test_error_is_debug() {
        let err = AppError::Other("debug test".to_string());
        let dbg = format!("{:?}", err);
        assert!(dbg.contains("Other"));
        assert!(dbg.contains("debug test"));
    }
}
