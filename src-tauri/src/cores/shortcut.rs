//! 全局快捷键核心逻辑：插件装配（事件处理）+ 启动时注册快捷键。
//!
//! 前端已整合 `@tauri-apps/plugin-global-shortcut`（权限 `global-shortcut:default` 就绪），
//! 当前未调用；如前端需动态注册/注销快捷键，直接使用 npm 包即可，无需后端改动。
//! Ctrl+Q 不直接 `app.exit`：经 `app:request-exit` 事件交前端按关闭行为偏好
//! （ask/quit/minimize）编排，与托盘退出菜单（tray.rs）同一路径。

use tauri::{Runtime, plugin::TauriPlugin};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use crate::cores::events::emit_request_exit;

/// 退出快捷键（Ctrl+Q）：setup 注册与 handler 校验共用同一组合，避免两处漂移
fn quit_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL), Code::KeyQ)
}

/// 构建全局快捷键插件：注册统一的快捷键事件处理器（退出应用）。
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            // 仅响应 Ctrl+Q 按下：前端动态注册的其他快捷键一律忽略，防止误触发退出
            if *shortcut == quit_shortcut() && event.state() == ShortcutState::Pressed {
                emit_request_exit(app);
            }
        })
        .build()
}

/// 注册全局快捷键：仅 Ctrl+Q（退出应用）。
/// 注册失败（如被其他应用占用）仅记录日志，不阻断启动。
/// @param app Tauri 应用实例
/// @returns 恒为 Ok；注册失败仅记录警告
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    match app.global_shortcut().register(quit_shortcut()) {
        Ok(()) => log::info!("[shortcut] registered Ctrl+Q to quit"),
        Err(error) => log::warn!("[shortcut] failed to register Ctrl+Q: {error}"),
    }
    Ok(())
}
