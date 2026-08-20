//! 系统配置 IPC 命令：薄层封装，核心逻辑位于 `crate::cores::config`。
//!
//! 读写约定：读取统一走 `get_config`（返回全部系统配置的类型化快照）；写入按配置项专项专用——
//! `set_locale` 写 locale（校验 + 同步 rust-i18n 运行时 + 重建托盘菜单），
//! `toggle_autostart` 切换 autostart、`toggle_tray` 切换系统托盘（先令操作系统生效再写回 config）。

use tauri::State;
use tauri_plugin_autostart::AutoLaunchManager;

use crate::cores::config::{ConfigKey, ConfigState, SystemConfig};
use crate::cores::locale::{Locale, SYSTEM_LOCALE};
use crate::cores::response::{CODE_ERROR, Response};

/// 读取全部系统配置：`invokeCommand("get_config")`（无参数）。
/// @param state 系统配置状态（经 setup 初始化）
/// @returns 类型化配置快照（locale/autostart/tray/notification）
#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Response<SystemConfig> {
    Response::ok(state.get_all())
}

/// 写入 locale 配置项并立即落盘：`invokeCommand("set_locale", { locale: "zh-CN" })`。
/// 接受「跟随系统」模式值（`system`，落盘哨兵、运行期解析跟随系统语言）或具体语言标签；
/// 非法值直接拒绝（不落盘）。落盘后同步切换 rust-i18n 运行时（生效语言为解析后的具体标签）。
/// 仅当 locale 实际发生变化时重建系统托盘菜单（相同值静默返回，不重复重建）。
/// @param app 应用句柄（Tauri 自动注入）
/// @param state 系统配置状态（经 setup 初始化）
/// @param locale 目标语言：`system` 或具体语言标签
/// @returns 当前生效的具体语言标签（前端 paraglide 运行时使用）
#[tauri::command]
pub fn set_locale(app: tauri::AppHandle, state: State<'_, ConfigState>, locale: String) -> Response<String> {
    // locale 为前后端 i18n 真相源：非法值直接拒绝写入，避免污染 config.json
    let system = locale == SYSTEM_LOCALE;
    if !system && Locale::new(&locale).is_none() {
        return Response::err(CODE_ERROR, "invalid locale");
    }

    // 与当前持久化值相同则直接返回（不落盘、不重建托盘菜单）
    let changed = state
        .get(ConfigKey::Locale)
        .and_then(|value| value.as_str().map(|current| current != locale.as_str()))
        .unwrap_or(true);
    if !changed {
        return Response::ok(state.resolve_locale().as_str().to_string());
    }

    if let Err(error) = state.set(ConfigKey::Locale, serde_json::Value::String(locale)) {
        return Response::err(error.code, error.message);
    }
    // 生效语言：system 模式解析跟随系统（匹配失败回退默认），具体标签原样
    let resolved = state.resolve_locale();
    rust_i18n::set_locale(resolved.as_str());
    crate::cores::tray::rebuild_menu(&app);
    #[cfg(target_os = "macos")]
    crate::cores::menu::rebuild_menu(&app);
    Response::ok(resolved.as_str().to_string())
}

/// 解析当前生效语言：`invokeCommand("resolve_locale")`（无参数）。
/// 存储值为「跟随系统」时返回跟随系统解析后的具体语言标签（系统不可得回退默认语言）；
/// 具体标签原样返回。供前端 initLocale 同步 paraglide 运行时使用。
/// @param state 系统配置状态（经 setup 初始化）
/// @returns 当前生效的具体语言标签
#[tauri::command]
pub fn resolve_locale(state: State<'_, ConfigState>) -> Response<String> {
    Response::ok(state.resolve_locale().as_str().to_string())
}

/// 切换自动启动状态：`invokeCommand("toggle_autostart")`。
/// 读 config 当前值取反 → 先令操作系统生效（失败不落盘）→ 写回 config。
/// @param config 系统配置状态（经 setup 初始化）
/// @param manager 插件管理的自动启动管理器（经 plugin init 注入）
/// @returns 切换后的 autostart 值；操作系统同步失败时返回错误码
#[tauri::command]
pub fn toggle_autostart(config: State<'_, ConfigState>, manager: State<'_, AutoLaunchManager>) -> Response<bool> {
    let enabled = config.read_bool(ConfigKey::Autostart, false);
    let enabled = !enabled;

    // 先 OS 生效，失败直接返回（不写回 config，避免两侧不一致）
    let result = if enabled { manager.enable() } else { manager.disable() };
    if let Err(error) = result {
        return Response::err(CODE_ERROR, error.to_string());
    }

    let value = serde_json::Value::Bool(enabled);
    if let Err(error) = config.set(ConfigKey::Autostart, value) {
        return Response::err(error.code, error.message);
    }
    Response::ok(enabled)
}

/// 切换系统托盘状态：`invokeCommand("toggle_tray")`。
/// 读 config 当前值取反 → 先设置托盘显隐（失败不落盘）→ 写回 config。
/// @param app 应用句柄（Tauri 自动注入）
/// @param config 系统配置状态（经 setup 初始化）
/// @returns 切换后的 tray 值；显隐设置失败时返回错误码
#[tauri::command]
pub fn toggle_tray(app: tauri::AppHandle, config: State<'_, ConfigState>) -> Response<bool> {
    let enabled = config.read_bool(ConfigKey::Tray, true);
    let enabled = !enabled;

    // 先设置托盘显隐，失败直接返回（不写回 config，避免两侧不一致）
    if let Err(error) = crate::cores::tray::set_visible(&app, enabled) {
        return Response::err(CODE_ERROR, error.to_string());
    }

    let value = serde_json::Value::Bool(enabled);
    if let Err(error) = config.set(ConfigKey::Tray, value) {
        return Response::err(error.code, error.message);
    }
    Response::ok(enabled)
}

/// 切换系统通知开关：`invokeCommand("toggle_notification")`。
/// 纯配置切换（无 OS 副作用）：读 config 当前值取反 → 写回 config。
/// @param config 系统配置状态（经 setup 初始化）
/// @returns 切换后的 notification 值
#[tauri::command]
pub fn toggle_notification(config: State<'_, ConfigState>) -> Response<bool> {
    let enabled = config.read_bool(ConfigKey::Notification, false);
    let enabled = !enabled;

    let value = serde_json::Value::Bool(enabled);
    if let Err(error) = config.set(ConfigKey::Notification, value) {
        return Response::err(error.code, error.message);
    }
    Response::ok(enabled)
}

/// 切换窗口状态记忆开关：`invokeCommand("toggle_window_state")`。
/// 纯配置切换（无 OS 副作用）：读 config 当前值取反 → 写回 config；
/// 生效时机在下次启动的 `cores::window_state::setup` 门控（关闭则不恢复窗口状态），
/// 退出时的状态记录为插件内置行为，不受开关影响（详见 window_state 模块文档）。
/// @param config 系统配置状态（经 setup 初始化）
/// @returns 切换后的 window_state 值
#[tauri::command]
pub fn toggle_window_state(config: State<'_, ConfigState>) -> Response<bool> {
    let enabled = config.read_bool(ConfigKey::WindowState, false);
    let enabled = !enabled;

    let value = serde_json::Value::Bool(enabled);
    if let Err(error) = config.set(ConfigKey::WindowState, value) {
        return Response::err(error.code, error.message);
    }
    Response::ok(enabled)
}

/// 恢复全部系统配置为默认值：`invokeCommand("reset_config")`。
/// 默认值语义：autostart=false、tray=true、notification=false、window_state=false、
/// locale=`system`（跟随系统语言，fresh-install 语义，运行期解析 + 回退默认语言）。
/// 编排（同 toggle 系列）：先 OS 生效（autostart 关闭 / 托盘恢复显示，失败不落盘）→
/// 全部 key 经 `set_many` 单次落盘（失败整体回滚，避免部分重置）→
/// locale 变更时同步 rust-i18n 运行时（解析后的生效语言）与托盘/macOS 菜单重建。
/// @param app 应用句柄（Tauri 自动注入）
/// @param config 系统配置状态（经 setup 初始化）
/// @param manager 插件管理的自动启动管理器（经 plugin init 注入）
/// @returns 重置后的完整配置快照；OS 同步失败时返回错误码
#[tauri::command]
pub fn reset_config(
    app: tauri::AppHandle,
    config: State<'_, ConfigState>,
    manager: State<'_, AutoLaunchManager>,
) -> Response<SystemConfig> {
    // 先 OS 生效，失败直接返回（不写回 config，避免两侧不一致）
    if config.read_bool(ConfigKey::Autostart, false)
        && let Err(error) = manager.disable()
    {
        return Response::err(CODE_ERROR, error.to_string());
    }
    if !config.read_bool(ConfigKey::Tray, true)
        && let Err(error) = crate::cores::tray::set_visible(&app, true)
    {
        return Response::err(CODE_ERROR, error.to_string());
    }

    let values = [
        (ConfigKey::Autostart, serde_json::Value::Bool(false)),
        (ConfigKey::Tray, serde_json::Value::Bool(true)),
        (ConfigKey::Notification, serde_json::Value::Bool(false)),
        (ConfigKey::WindowState, serde_json::Value::Bool(false)),
        (ConfigKey::Locale, serde_json::Value::String(SYSTEM_LOCALE.to_string())),
    ];
    if let Err(error) = config.set_many(&values) {
        return Response::err(error.code, error.message);
    }

    let resolved = config.resolve_locale();
    rust_i18n::set_locale(resolved.as_str());
    crate::cores::tray::rebuild_menu(&app);
    #[cfg(target_os = "macos")]
    crate::cores::menu::rebuild_menu(&app);
    Response::ok(config.get_all())
}
