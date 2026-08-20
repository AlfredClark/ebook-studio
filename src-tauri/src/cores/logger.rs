//! 日志插件装配：统一配置前后端共用的 tauri-plugin-log。
//!
//! 前后端共用同一套日志：Rust 侧用 `log` crate 宏（log::info! 等），
//! 前端经 `@tauri-apps/plugin-log`（封装于 src/libs/logger/）写入同一链路。

use tauri::{Runtime, plugin::TauriPlugin};

/// 构建日志插件。
///
/// - 级别：dev 全量 Trace（调试信息完整），release Info（生产减噪）
/// - 目标：stdout（终端）+ LogDir（应用日志目录，活跃文件 app.log）+ Webview（devtools 控制台）
/// - 轮转：1MB/文件，KeepSome(10) 最多保留 10 个文件；归档名含日期时间（如 app_2026-08-05_14-30-22.log）
/// - 会话滚动：FileOpenStrategy::Rotate，每次启动归档上一会话
/// - 时区：本地时区（阅读友好）
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_log::Builder::new()
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(10))
        .file_open_strategy(tauri_plugin_log::FileOpenStrategy::Rotate)
        .max_file_size(1_000_000)
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .targets([
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                file_name: Some("app".to_string()),
            }),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
        ])
        .build()
}
