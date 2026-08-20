pub mod clean;
pub mod config;
pub mod env;

/// 汇总所有需要暴露给前端的 IPC 命令：lib.rs 的 invoke_handler 只需调用一次本宏。
/// 新增命令后在此列表追加，lib.rs 无需改动。
macro_rules! invoke_handlers {
    () => {
        tauri::generate_handler![
            $crate::commands::clean::list_txts,
            $crate::commands::clean::copy_txt,
            $crate::commands::clean::get_txt_detail,
            $crate::commands::clean::delete_txt,
            $crate::commands::config::get_config,
            $crate::commands::config::set_locale,
            $crate::commands::config::resolve_locale,
            $crate::commands::config::toggle_autostart,
            $crate::commands::config::toggle_tray,
            $crate::commands::config::toggle_notification,
            $crate::commands::config::toggle_window_state,
            $crate::commands::config::reset_config,
            $crate::commands::env::is_always_on_top_supported,
        ]
    };
}
pub(crate) use invoke_handlers;
