//! Tauri 应用核心库。
//!
//! 模块拆分：`cores/` 为核心逻辑（含初始化 setup），`commands/` 为 IPC 命令薄层；
//! 本文件仅声明模块、组装 Builder 与注册命令。
//! `src/main.rs` 仅作为二进制入口委托调用 `run()`。

mod commands;
mod cores;
mod features;

use commands::invoke_handlers;

// 初始化 rust-i18n：加载 src-tauri/locales 下的消息源，缺失翻译回退 en
rust_i18n::i18n!("locales", fallback = "en");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 必须在 Builder 创建前调用：WebKit 环境 workaround
    cores::env::init_env();
    // 全局 panic hook：panic 写入日志链路（logger 插件初始化前的早期 panic 仅 stderr 输出）
    cores::panic::init_hook();
    tauri::Builder::default()
        // 单实例插件置于链首：尽早注册单例锁，避免窗口建好后回调竞态
        .plugin(cores::instance::plugin())
        .plugin(cores::config::plugin())
        .plugin(cores::autostart::plugin())
        .plugin(cores::logger::plugin())
        .plugin(cores::shortcut::plugin())
        .plugin(cores::window_state::plugin())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_system_fonts::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(cores::setup_cores)
        .invoke_handler(invoke_handlers!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
