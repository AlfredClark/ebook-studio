//! 环境信息命令：查询当前运行环境的能力。

use crate::cores::env;
use crate::cores::response::Response;

/// IPC：查询当前环境是否支持窗口置顶（Linux Wayland 下 GTK keep_above 无效，
/// 前端据此隐藏置顶按钮）。
/// 前端调用示例：`invokeCommand("is_always_on_top_supported")`
/// @returns 是否支持窗口置顶
#[tauri::command]
pub fn is_always_on_top_supported() -> Response<bool> {
    Response::ok(env::is_always_on_top_supported())
}
