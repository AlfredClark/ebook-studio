//! 演示业务：问候语生成（IPC 调用示例，无真实业务状态）。

use rust_i18n::t;

use crate::cores::response::AppResult;

/// 生成问候语，经 rust-i18n 本地化（locale 由 config.json 驱动，见 cores/config.rs）。
/// @param name 用户输入的名称
/// @returns 本地化问候语
pub(crate) fn greet(name: &str) -> AppResult<String> {
    Ok(t!("demo.greet", name = name).to_string())
}
