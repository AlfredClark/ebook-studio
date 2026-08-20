//! 窗口状态记忆核心逻辑：经 tauri-plugin-window-state 记录/恢复窗口尺寸、位置与最大化状态。
//!
//! 真相源约定：config.json 为窗口状态记忆开关的唯一真相源（`window_state` key）。
//! 设计要点：
//! - 插件经 `skip_initial_state("main")` 关闭自动恢复（恢复改由本模块 setup 按配置门控），
//!   跟踪与退出保存（RunEvent::Exit 写盘）为插件内置行为，不受开关影响——关闭期间
//!   仍会记录，但启动时因开关关闭不恢复，重新开启后恢复到最近一次记录（"暂停记忆"语义）。
//! - 已知边界：Wayland 下位置恢复无效（合成器决定窗口摆放，xdg-shell 无 set_position），
//!   尺寸/最大化恢复正常；X11 会话三项完整恢复。强杀进程（无 Exit 事件）不落盘。

use tauri::{Manager, Runtime, plugin::TauriPlugin};
use tauri_plugin_window_state::{StateFlags, WindowExt};

use crate::cores::config::{ConfigKey, ConfigState};

/// 装配 window-state 插件：关闭主窗口的自动恢复（恢复经 setup 按配置门控），
/// 跟踪与退出保存保留。
/// @returns 插件实例
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_window_state::Builder::default()
        .skip_initial_state("main")
        .build()
}

/// 窗口状态记忆初始化：按 config 持久化的 window_state 值决定是否恢复主窗口状态。
/// 开启 → 恢复上次记录的尺寸/位置/最大化；关闭 → 保持默认（tauri.conf.json 配置）。
/// 恢复须在窗口创建后调用（setup 阶段主窗口已存在），且晚于插件 setup 完成磁盘缓存加载。
/// 恢复失败不阻断启动：窗口记忆属可恢复能力，按约定 warn 后继续。
/// @param app Tauri 应用实例
/// @returns 恒为 Ok（恢复失败仅记录 warn，不阻断启动）
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let enabled = app.state::<ConfigState>().read_bool(ConfigKey::WindowState, false);
    if !enabled {
        log::info!("[window-state] disabled, skip restoring window state");
        return Ok(());
    }
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("[window-state] main window not found, skip restoring window state");
        return Ok(());
    };
    // 恢复失败（如 Wayland 位置恢复异常）不阻断启动，warn 后继续
    if let Err(error) = window.restore_state(StateFlags::all()) {
        log::warn!("[window-state] restore failed: {error}");
        return Ok(());
    }
    // 插件按保存的 visible 决定是否 show/focus：隐藏状态下退出（托盘隐藏/最小化到托盘后退出）
    // 会落盘 visible=false，导致下次启动插件跳过 show/set_focus、窗口无焦点。
    // 窗口由 tauri.conf 默认 visible=true 创建可见，这里显式补齐 show+focus（show 对可见窗口为 no-op）。
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}
