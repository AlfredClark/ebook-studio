//! 前端事件契约层：事件名常量 + payload 类型 + 类型化 emit 封装。
//!
//! 设计要点：
//! - 与前端 `src/libs/events` 镜像对齐：事件名、payload 类型两端各维护一份，
//!   经命名/序列化约定保持一致（serde lowercase ↔ TS 字符串字面量）。
//! - 静态契约模块（无 plugin()/setup()，同 response.rs），不承载业务逻辑。
//! - 传输机制决策：后端 → 前端单次通知用事件；流式/进度/批量用 Channel；
//!   前端 → 后端请求用 command，不经事件。事件 ≤5 个维持轻量契约层，
//!   超过后升级为全量契约（payload 全 serde 结构体 + 事件清单文档），不建自建事件总线。

// 契约符号由平台条件模块消费（menu.rs 仅 macOS 编译）：非 macOS 平台下无引用属正常现象，
// 故允许 dead_code（勿删契约符号——两端镜像的 API 面，非死代码）
#![allow(dead_code)]

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

/// 菜单导航事件（payload 为 `MenuPage`，前端统一监听切换路由）
pub const EVENT_MENU_NAVIGATE: &str = "menu:navigate";

/// 退出请求事件（托盘退出菜单 / Ctrl+Q 统一经此事件交前端按关闭行为编排，payload 为 `()`）
pub const EVENT_REQUEST_EXIT: &str = "app:request-exit";

/// 菜单导航页面标识（serde lowercase 序列化，与前端 `MenuPage` 类型对齐）
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MenuPage {
    /// 首页
    Home,
    /// 设置页
    Settings,
    /// 关于页
    About,
}

/// 发送菜单导航事件：前端统一监听切换路由。
/// emit 失败仅记录日志（如前端尚未就绪），菜单交互不受影响。
/// @param app 应用句柄
/// @param page 目标页面标识
pub fn emit_menu_navigate<R: Runtime>(app: &AppHandle<R>, page: MenuPage) {
    if let Err(error) = app.emit(EVENT_MENU_NAVIGATE, page) {
        log::error!("[events] failed to emit {EVENT_MENU_NAVIGATE}: {error}");
    }
}

/// 发送退出请求事件：前端监听后按关闭行为偏好（ask/quit/minimize）编排退出流程，
/// 不再由后端直接 `app.exit` 绕过偏好。
/// emit 失败仅记录日志（如前端尚未就绪），调用方（托盘/快捷键）流程不受影响。
/// @param app 应用句柄
pub fn emit_request_exit<R: Runtime>(app: &AppHandle<R>) {
    if let Err(error) = app.emit(EVENT_REQUEST_EXIT, ()) {
        log::error!("[events] failed to emit {EVENT_REQUEST_EXIT}: {error}");
    }
}
