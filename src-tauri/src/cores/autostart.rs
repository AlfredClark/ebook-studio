//! 自动启动核心逻辑：插件装配 + 启动时将 config.json 的 autostart 偏好同步到操作系统。
//!
//! 真相源约定：config.json 为 autostart 的唯一真相源，本模块负责在启动时
//! 按持久化值 apply 到 OS（enable/disable）；前端切换经 `toggle_autostart` 命令，
//! 先 OS 生效再写回 config，保证两者始终一致。

use tauri::{Manager, Runtime, plugin::TauriPlugin};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::cores::config::{ConfigKey, ConfigState};

/// 构建自动启动插件：macOS 采用 LaunchAgent 方式，不附加额外启动参数。
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)
}

/// 自动启动初始化：读取 config 持久化的 autostart 值并同步到操作系统。
///
/// 同步失败（如写入 ~/.config/autostart 失败）不阻断应用启动，记录后继续。
/// @param app Tauri 应用实例
/// @returns 恒为 Ok；同步失败仅记录日志
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let enabled = app.state::<ConfigState>().read_bool(ConfigKey::Autostart, false);
    let manager = app.autolaunch();
    let result = if enabled { manager.enable() } else { manager.disable() };
    if let Err(error) = result {
        log::warn!("[autostart] failed to sync autostart state: {error}");
    }
    Ok(())
}
