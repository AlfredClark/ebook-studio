//! 演示命令：IPC 调用示例（薄层：调 features → 经 From 转 Response）。

use crate::cores::response::Response;
use crate::features;

/// IPC 命令示例：前端通过 `invokeCommand("greet", { name })` 调用。
/// @param name 用户输入的名称
/// @returns 问候语
#[tauri::command]
pub fn greet(name: &str) -> Response<String> {
    features::demo::greet(name).into()
}
