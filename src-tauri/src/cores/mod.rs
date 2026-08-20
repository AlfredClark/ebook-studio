pub mod autostart;
pub mod config;
pub mod env;
pub mod events;
pub mod instance;
pub mod locale;
pub mod logger;

#[cfg(target_os = "macos")]
pub mod menu;
pub mod panic;
pub mod response;
pub mod shortcut;
pub mod tray;
pub mod window_state;

/// 整合 cores 下所有模块的初始化 setup，供 lib.rs 统一调用。
/// @param app Tauri 应用实例
/// @returns 任一模块初始化失败时返回错误
pub fn setup_cores(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    config::setup(app)?;
    autostart::setup(app)?;
    tray::setup(app)?;
    shortcut::setup(app)?;
    #[cfg(target_os = "macos")]
    menu::setup(app)?;
    window_state::setup(app)?;
    Ok(())
}
