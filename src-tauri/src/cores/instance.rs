//! 单实例核心逻辑：防多开；多开时聚焦已有实例主窗口。
//!
//! 机制（Linux）：首个实例注册 D-Bus 名 `{identifier}.SingleInstance`；
//! 第二实例启动时发现名字被占，回调经 D-Bus 于**首个实例进程内**执行
//! （argv/cwd 随回调传入，此处忽略），随后第二实例自行退出。

use tauri::{Manager, Runtime, plugin::TauriPlugin};

/// 构建单实例插件：第二实例触发回调（运行于首个实例进程）后自行退出。
/// 回调仅聚焦已有实例主窗口，argv/cwd 不参与日志（避免噪声）。
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        log::info!("[instance] secondary instance detected, focusing main window");
        // 聚焦已有实例主窗口（取消最小化 + 显示 + 聚焦）
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    })
}
