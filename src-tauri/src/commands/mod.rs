pub mod build;
pub mod clean;
pub mod config;
pub mod env;
pub mod inspect;
pub mod projects;
pub mod split;

/// 汇总所有需要暴露给前端的 IPC 命令：lib.rs 的 invoke_handler 只需调用一次本宏。
/// 新增命令后在此列表追加，lib.rs 无需改动。
macro_rules! invoke_handlers {
    () => {
        tauri::generate_handler![
            $crate::commands::config::get_config,
            $crate::commands::config::set_locale,
            $crate::commands::config::resolve_locale,
            $crate::commands::config::toggle_autostart,
            $crate::commands::config::toggle_tray,
            $crate::commands::config::toggle_notification,
            $crate::commands::config::toggle_window_state,
            $crate::commands::config::reset_config,
            $crate::commands::env::is_always_on_top_supported,
            $crate::commands::projects::list_projects,
            $crate::commands::projects::create_project,
            $crate::commands::projects::update_project,
            $crate::commands::projects::get_project,
            $crate::commands::projects::delete_project,
            $crate::commands::projects::batch_delete_projects,
            $crate::commands::projects::resolve_project_asset,
            $crate::commands::projects::get_file_stats,
            $crate::commands::projects::read_image_as_data_url,
            $crate::commands::inspect::inspect_content,
            $crate::commands::inspect::reorder_chapters,
            $crate::commands::inspect::get_content_path,
            $crate::commands::split::get_split_content,
            $crate::commands::split::split_content,
            $crate::commands::split::save_split_chapter,
            $crate::commands::clean::filter_clean,
            $crate::commands::build::get_build,
            $crate::commands::build::build_epub,
            $crate::commands::build::remove_build,
            $crate::commands::build::read_build_file,
            $crate::commands::build::write_build_file,
            $crate::commands::build::get_build_path,
        ]
    };
}
pub(crate) use invoke_handlers;
