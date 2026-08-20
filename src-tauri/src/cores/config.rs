//! 系统级配置核心逻辑：config.json 的初始化与读写，经 Tauri State 缓存避免重复读文件。
//!
//! 与前端 UI 偏好（localStorage stores 模块）按配置归属分层：
//! 本模块管理系统级配置（如 locale），前端偏好持久化归 stores 模块。

use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::{Manager, Runtime, plugin::TauriPlugin};
use tauri_plugin_store::{Store, StoreExt};

use crate::cores::locale::{Locale, SYSTEM_LOCALE};
use crate::cores::response::{AppError, AppResult};

/// 构建 store 插件（config.json 持久化底层，经本模块统一装配）。
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_store::Builder::new().build()
}

/// config.json 文件名（存放于应用数据目录）
const FILE_NAME: &str = "config.json";

/// 系统级配置完整快照（get_config 返回值；字段与 config.json 的 key 一一对应）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    /// 界面语言标签
    pub locale: String,
    /// 开机自启开关
    pub autostart: bool,
    /// 系统托盘开关
    pub tray: bool,
    /// 系统通知开关
    pub notification: bool,
    /// 窗口状态记忆开关
    pub window_state: bool,
}

/// config.json 配置项 key（变体与 `SystemConfig` 字段一一对应，跨层引用经枚举保证拼写正确）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigKey {
    /// 界面语言（对应 SystemConfig::locale）
    Locale,
    /// 开机自启（对应 SystemConfig::autostart）
    Autostart,
    /// 系统托盘（对应 SystemConfig::tray）
    Tray,
    /// 系统通知（对应 SystemConfig::notification）
    Notification,
    /// 窗口状态记忆（对应 SystemConfig::window_state）
    WindowState,
}

impl ConfigKey {
    /// 持久化键名（config.json 字符串；历史兼容既有配置文件，值不可变更）
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Locale => "locale",
            Self::Autostart => "autostart",
            Self::Tray => "tray",
            Self::Notification => "notification",
            Self::WindowState => "window_state",
        }
    }
}

/// 默认语言标签（与前端 paraglide baseLocale 一致）
const DEFAULT_LOCALE: &str = "en";

/// 解析初始 locale：首次运行/配置缺失时优先跟随系统语言
/// （完整标签或主语言子标签精确匹配，经 `tauri_plugin_os::locale()` 获取），
/// 系统语言不可得或不匹配时回退默认语言。
/// 为什么放这里：系统语言检测为启动期初始化行为，locale 校验归 Locale 类型（from_system）；
/// `pub(crate)` 供命令层复用（reset_config 的 locale 默认值语义 = 跟随系统语言）。
/// @returns 校验通过的 locale（回退后恒为合法值）
pub(crate) fn initial_locale() -> Locale {
    tauri_plugin_os::locale()
        .and_then(|value| Locale::from_system(&value))
        .unwrap_or_else(|| Locale::new(DEFAULT_LOCALE).expect("default locale must be available"))
}

/// 从 store 加载配置：locale 缺失/非法时回退默认并写回修复；
/// 布尔配置项（autostart/tray/notification）逐键校验，损坏值同样修复落盘。
/// locale 缺失时写入「跟随系统」模式值（`system`）——首启即跟随系统语言，
/// 运行期经 `read_locale` 解析具体语言（见 Locale::from_system）。
/// @param store plugin-store 的 store 引用
/// @returns 加载结果；落盘修复失败时返回错误
pub fn load(store: &Store<tauri::Wry>) -> AppResult<Locale> {
    let mut repaired = false;

    let locale = match store.get(ConfigKey::Locale.as_str()) {
        Some(serde_json::Value::String(value)) if value == SYSTEM_LOCALE => Some(initial_locale()),
        Some(serde_json::Value::String(value)) => Locale::new(&value),
        _ => None,
    };
    let locale = locale.unwrap_or_else(|| {
        // 初次运行/配置缺失：写入「跟随系统」模式值（system），解析留待运行期（开箱即用）
        store.set(
            ConfigKey::Locale.as_str().to_string(),
            serde_json::Value::String(SYSTEM_LOCALE.to_string()),
        );
        repaired = true;
        initial_locale()
    });

    // 读取方（autostart/tray/setup）各自带回退，此处仅兜底修复文件内的损坏值
    load_bool(store, ConfigKey::Autostart, false, &mut repaired);
    load_bool(store, ConfigKey::Tray, true, &mut repaired);
    load_bool(store, ConfigKey::Notification, false, &mut repaired);
    load_bool(store, ConfigKey::WindowState, false, &mut repaired);

    // 仅当存在修复时落盘，健康文件不写盘
    if repaired {
        store.save()?;
    }
    Ok(locale)
}

/// 读取布尔配置项；缺失/非布尔时回退默认值、写回修复并标记。
/// @param store plugin-store 的 store 引用
/// @param key 配置项 key
/// @param default 缺失/损坏时的默认值
/// @param repaired 修复标记（发生回退时置为 true）
fn load_bool(store: &Store<tauri::Wry>, key: ConfigKey, default: bool, repaired: &mut bool) {
    if store.get(key.as_str()).and_then(|value| value.as_bool()).is_none() {
        store.set(key.as_str().to_string(), serde_json::Value::Bool(default));
        *repaired = true;
    }
}

/// 系统配置状态：持有 plugin-store 的 store 缓存，命令经 `State` 注入读取
pub struct ConfigState {
    store: Arc<Store<tauri::Wry>>,
}

impl ConfigState {
    /// 读取全部系统配置（类型化快照；缺失/损坏条目回退默认值）。
    /// locale 字段为存储模式值（`system` 或具体语言标签，供前端 Select 展示选中态）。
    /// @returns 配置快照（locale/autostart/tray/notification/window_state）
    pub fn get_all(&self) -> SystemConfig {
        SystemConfig {
            locale: self.read_stored_locale(),
            autostart: self.read_bool(ConfigKey::Autostart, false),
            tray: self.read_bool(ConfigKey::Tray, true),
            notification: self.read_bool(ConfigKey::Notification, false),
            window_state: self.read_bool(ConfigKey::WindowState, false),
        }
    }

    /// 读取存储的 locale 模式值（`system` 或具体语言标签）。
    /// @returns 存储值；条目缺失/非法时回退跟随系统语言解析后的标签
    fn read_stored_locale(&self) -> String {
        match self.store.get(ConfigKey::Locale.as_str()) {
            Some(serde_json::Value::String(value)) => value,
            _ => initial_locale().as_str().to_string(),
        }
    }

    /// 解析当前生效语言：存储值为 `system` 时跟随系统语言（匹配失败回退默认语言），
    /// 具体标签原样返回；缺失/非法时同样回退跟随系统。
    /// @returns 校验通过的 locale（回退后恒为合法值）
    pub fn resolve_locale(&self) -> Locale {
        match self.store.get(ConfigKey::Locale.as_str()) {
            Some(serde_json::Value::String(value)) if value == SYSTEM_LOCALE => initial_locale(),
            Some(serde_json::Value::String(value)) => Locale::new(&value).unwrap_or_else(initial_locale),
            _ => initial_locale(),
        }
    }

    /// 读取布尔配置项：缺失/非布尔时回退默认值。
    /// @param key 配置项 key
    /// @param default 缺失/损坏时的默认值
    /// @returns 持久化的布尔值；缺失/损坏时返回默认值
    pub(crate) fn read_bool(&self, key: ConfigKey, default: bool) -> bool {
        self.store
            .get(key.as_str())
            .and_then(|value| value.as_bool())
            .unwrap_or(default)
    }

    /// 读取配置项。
    /// @param key 配置项 key
    /// @returns 配置值；条目不存在时返回 None
    pub fn get(&self, key: ConfigKey) -> Option<serde_json::Value> {
        self.store.get(key.as_str())
    }

    /// 写入配置项并立即落盘；落盘失败时回滚内存缓存，保证内存态与持久化一致。
    /// @param key 配置项 key
    /// @param value 配置值
    /// @returns 落盘失败时返回错误
    pub fn set(&self, key: ConfigKey, value: serde_json::Value) -> AppResult<()> {
        let key_str = key.as_str();
        let previous = self.store.get(key_str);
        self.store.set(key_str.to_string(), value);
        if let Err(error) = self.store.save() {
            // 回滚：恢复旧值，避免内存态与磁盘及已执行的副作用（如 OS 切换）不一致。
            // 配置 key 经 load 修复后恒存在，previous 恒为 Some（StoreHandle 无 remove，仅支持回写）
            if let Some(previous) = previous {
                self.store.set(key_str.to_string(), previous);
            }
            return Err(AppError::from(error));
        }
        Ok(())
    }

    /// 批量写入配置项并单次落盘；落盘失败时整体回滚内存缓存。
    /// 供全量重置（reset_config）等多 key 原子更新使用——逐 key `set` 会因
    /// 多次 save 导致中途失败时部分 key 已持久化（与「先 OS 生效再写回」的编排冲突）。
    /// @param values 配置项与值的切片（按切片顺序写入内存）
    /// @returns 落盘失败时返回错误（内存已回滚，磁盘保持旧值）
    pub fn set_many(&self, values: &[(ConfigKey, serde_json::Value)]) -> AppResult<()> {
        let previous: Vec<(&str, Option<serde_json::Value>)> = values
            .iter()
            .map(|(key, _)| (key.as_str(), self.store.get(key.as_str())))
            .collect();
        for (key, value) in values {
            self.store.set(key.as_str().to_string(), value.clone());
        }
        if let Err(error) = self.store.save() {
            // save 失败磁盘未变，仅需回滚内存；旧值缺失（不应发生，load 修复后恒存在）时保持新值
            for (key, value) in previous {
                if let Some(value) = value {
                    self.store.set(key.to_string(), value);
                }
            }
            return Err(AppError::from(error));
        }
        Ok(())
    }
}

/// config 初始化：加载 config.json 存入 Tauri State（缓存避免重复读文件），
/// 并将 locale 同步给 rust-i18n 运行时（locale 为前后端 i18n 的公共真相源）。
/// 文件损坏时不阻断启动：备份为 *.corrupt 后重建默认配置。
/// @param app Tauri 应用实例
/// @returns 初始化失败时返回错误
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let store = match app.store(FILE_NAME) {
        Ok(store) => store,
        Err(error) => {
            log::error!("[config] failed to load {FILE_NAME}: {error}, resetting");
            recover_corrupt_config(app);
            app.store(FILE_NAME)?
        }
    };
    // store 插件对损坏 JSON 静默吞错（build 内 load 错误被忽略），文件存在时用 reload 显式检测。
    // 路径解析与 tauri-plugin-store 的 resolve_store_path 一致（BaseDirectory::AppData），
    // 避免目录枚举漂移导致检测/备份找错文件（store 实际落盘于 AppData）
    let config_path = app.path().resolve(FILE_NAME, BaseDirectory::AppData)?;
    if config_path.exists() && store.reload().is_err() {
        log::error!("[config] config.json corrupted, resetting");
        recover_corrupt_config(app);
    }
    let config = load(&store)?;
    rust_i18n::set_locale(config.as_str());
    app.manage(ConfigState { store });
    Ok(())
}

/// config.json 损坏时：备份为 *.corrupt 后移除，令 store 插件重建空文件。
/// 配置仅为偏好，损坏不阻断启动；备份保留现场便于排查。
/// 路径解析与 tauri-plugin-store 一致（BaseDirectory::AppData），确保备份的是真实落盘文件。
fn recover_corrupt_config(app: &tauri::App) {
    let Ok(config_path) = app.path().resolve(FILE_NAME, BaseDirectory::AppData) else {
        return;
    };
    if !config_path.exists() {
        return;
    }
    if std::fs::rename(&config_path, config_path.with_extension("json.corrupt")).is_err() {
        let _ = std::fs::remove_file(&config_path);
    }
}
