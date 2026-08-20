//! 应用菜单栏核心逻辑（macOS 专属）：系统菜单栏 + 导航三页 + Edit/Quit 预设。
//!
//! 平台策略：本应用窗口为 `decorations: false` 自绘标题栏，Windows/Linux 上窗口菜单
//! 会与自绘标题栏冲突（且托盘 + 全局快捷键已覆盖常用操作），故菜单仅在 macOS 编译
//! 启用（`#[cfg(target_os = "macos")]`）——macOS 菜单渲染在系统全局菜单栏，与窗口无关。
//! 设计要点：
//! - 导航组（首页/设置/关于）：文案经 rust-i18n 本地化，点击 emit `menu:navigate`
//!   事件（payload 为目标页标识），前端统一监听切换路由；Cmd+1/2/3 加速器。
//! - Edit 组（Undo/Redo/Cut/Copy/Paste/Select All）：tauri 预设项，补齐 WKWebView
//!   编辑命令（无 Edit 菜单时 Cmd+C/V 在输入框可能失效）。
//! - App 菜单 Quit：预设项（Cmd+Q）；与全局快捷键 Ctrl+Q（shortcut.rs）修饰键不同，不冲突。
//! - 语言切换经 `rebuild_menu` 重建（与 tray::rebuild_menu 同模式）；on_menu_event
//!   为 AppHandle 级注册，重建菜单不丢失事件绑定。
//! - 已知边界：预设项文本为英文（tauri 预设固定文案），导航组本地化。

use rust_i18n::t;
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem, SubmenuBuilder};

use crate::cores::events::{MenuPage, emit_menu_navigate};

/// 导航菜单项 id
const MENU_HOME: &str = "menu-home";
/// 导航菜单项 id
const MENU_PROJECTS: &str = "menu-projects";
/// 导航菜单项 id
const MENU_SETTINGS: &str = "menu-settings";
/// 导航菜单项 id
const MENU_ABOUT: &str = "menu-about";

/// 构建应用菜单：App（Quit）→ Edit（编辑预设）→ 导航（首页/设置/关于）。
/// App 菜单标题占位（macOS 首菜单标题由系统替换为应用名）。
/// @param app 应用句柄
/// @returns 菜单构建结果
fn build_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let app_menu = SubmenuBuilder::new(app, "app")
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    let home = MenuItem::with_id(app, MENU_HOME, t!("menu.home"), true, Some("Cmd+1"))?;
    let projects = MenuItem::with_id(app, MENU_PROJECTS, t!("menu.projects"), true, Some("Cmd+2"))?;
    let settings = MenuItem::with_id(app, MENU_SETTINGS, t!("menu.settings"), true, Some("Cmd+3"))?;
    let about = MenuItem::with_id(app, MENU_ABOUT, t!("menu.about"), true, Some("Cmd+4"))?;

    let navigate_menu = SubmenuBuilder::new(app, t!("menu.navigate"))
        .item(&home)
        .item(&projects)
        .item(&settings)
        .item(&about)
        .build()?;

    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&edit_menu)
        .item(&navigate_menu)
        .build()
}

/// 注册菜单事件处理器（AppHandle 级，仅需一次；菜单重建不影响）。
/// 导航三项经 `events::emit_menu_navigate` 发送（payload 为 `MenuPage`），由前端统一切换路由；
/// Quit/Edit 为预设项自带行为，无需处理。
/// @param app 应用句柄
fn register_events<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    app.on_menu_event(|app, event| {
        let page = match event.id().as_ref() {
            MENU_HOME => Some(MenuPage::Home),
            MENU_PROJECTS => Some(MenuPage::Projects),
            MENU_SETTINGS => Some(MenuPage::Settings),
            MENU_ABOUT => Some(MenuPage::About),
            _ => None,
        };
        if let Some(page) = page {
            // emit 失败仅记录日志（如前端尚未就绪），菜单交互不受影响
            emit_menu_navigate(app, page);
        }
    });
}

/// 应用菜单初始化：注册菜单事件 + 构建并设置菜单。
/// 构建/设置失败仅记录日志不阻断启动（菜单属可恢复能力）。
/// @param app Tauri 应用实例
/// @returns 恒为 Ok
#[cfg(target_os = "macos")]
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    register_events(app.handle());
    rebuild_menu(app.handle());
    log::info!("[menu] application menu set up");
    Ok(())
}

/// 按当前语言重建应用菜单（语言切换时调用，与 tray::rebuild_menu 同模式）。
/// @param app 应用句柄
pub fn rebuild_menu(app: &tauri::AppHandle) {
    match build_menu(app) {
        Ok(menu) => {
            if let Err(error) = app.set_menu(menu) {
                log::error!("[menu] failed to set menu: {error}");
            }
        }
        Err(error) => log::error!("[menu] failed to build menu: {error}"),
    }
}
