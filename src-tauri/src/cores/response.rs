//! 前后端统一响应协议：所有 IPC 命令返回 `Response<T>` 包裹，invoke 永不 reject。

/// 成功业务码
pub const CODE_OK: i32 = 0;
/// 通用内部错误码
pub const CODE_ERROR: i32 = 1;

/// 统一响应包装：code=0 成功（data 有值）；code!=0 失败（data 为 None）
#[derive(Debug, Clone, serde::Serialize)]
pub struct Response<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> Response<T> {
    /// 构造成功响应。
    /// @param data 业务数据
    /// @returns 成功响应
    pub fn ok(data: T) -> Self {
        Self {
            code: CODE_OK,
            message: String::new(),
            data: Some(data),
        }
    }

    /// 构造失败响应。
    /// @param code 业务错误码
    /// @param message 错误信息
    /// @returns 失败响应
    pub fn err(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

/// 统一错误类型：core 层返回 `AppResult<T>`，command 层经 `From` 转换为 `Response<T>`
#[derive(Debug)]
pub struct AppError {
    pub code: i32,
    pub message: String,
}

impl AppError {
    /// 构造错误。
    /// @param code 业务错误码
    /// @param message 错误信息
    /// @returns 错误实例
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

/// core 层统一返回类型
pub type AppResult<T> = Result<T, AppError>;

impl From<tauri_plugin_store::Error> for AppError {
    fn from(error: tauri_plugin_store::Error) -> Self {
        Self::new(CODE_ERROR, error.to_string())
    }
}

impl<T> From<AppResult<T>> for Response<T> {
    fn from(result: AppResult<T>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(error) => Self::err(error.code, error.message),
        }
    }
}
