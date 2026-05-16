use serde::Serialize;

/// 后端错误类型
///
/// 统一向前端透传字符串错误信息；vfox CLI 自身的 stderr 信息已足够清晰，
/// 不做二次包装。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("vfox 未安装或不在 PATH 中")]
    VfoxNotFound,

    #[error("vfox CLI 执行失败 (exit={code}): {stderr}")]
    VfoxFailed { code: i32, stderr: String },

    #[error("无法解析 vfox 输出: {0}")]
    ParseError(String),

    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML 错误: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

// Tauri 命令的错误必须实现 Serialize
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
