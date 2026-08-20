//! 语言标签类型：约束 locale 取值，避免非法值写入 config.json 或传给 rust-i18n 运行时。

use std::fmt;

/// 「跟随系统」模式哨兵值：config.json 的 locale 存此值时表示语言跟随系统
/// （运行期经 `Locale::from_system` 解析 + 回退默认语言），而非固定语言标签。
pub(crate) const SYSTEM_LOCALE: &str = "system";

/// 语言标签：仅允许 rust-i18n 已注册的 locale（en / zh-CN）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale(String);

impl Locale {
    /// 校验并构造；值不在可用 locale 列表时返回 None。
    /// @param value 语言标签（如 "en"、"zh-CN"）
    /// @returns 校验通过的 Locale
    pub fn new(value: &str) -> Option<Self> {
        rust_i18n::available_locales!()
            .iter()
            .any(|available| available == value)
            .then(|| Self(value.to_string()))
    }

    /// 从系统语言标签（BCP-47，如 "zh-CN"、"en-US"）解析可用 locale：
    /// 先精确匹配完整标签，再按主语言子标签精确匹配；均无命中返回 None。
    /// 归一化对齐 sys_locale：`_` 转 `-`、剥离 `.` 编码与 `@` 修饰后缀、大小写不敏感。
    /// @param value 系统语言标签（可为 POSIX 形态，如 "zh_CN.UTF-8"）
    /// @returns 匹配到的 Locale
    pub fn from_system(value: &str) -> Option<Self> {
        let normalized = value
            .split(['.', '@'])
            .next()
            .unwrap_or_default()
            .replace('_', "-")
            .to_lowercase();
        if normalized.is_empty() {
            return None;
        }
        let available = rust_i18n::available_locales!();
        // 完整标签精确匹配（如 "zh-cn" → "zh-CN"）
        if let Some(matched) = available.iter().find(|available| available.eq_ignore_ascii_case(&normalized)) {
            return Self::new(matched);
        }
        // 主语言子标签精确匹配（如 "zh-tw" → "zh-CN"、"en-us" → "en"）
        let language = normalized.split('-').next()?;
        let matched = available.iter().find(|available| {
            available
                .split('-')
                .next()
                .is_some_and(|lang| lang.eq_ignore_ascii_case(language))
        })?;
        Self::new(matched)
    }

    /// 原始语言标签字符串。
    /// @returns 语言标签（如 "en"）
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 完整标签精确匹配，大小写不敏感。
    #[test]
    fn from_system_exact_match() {
        assert_eq!(Locale::from_system("zh-CN"), Locale::new("zh-CN"));
        assert_eq!(Locale::from_system("ZH-cn"), Locale::new("zh-CN"));
        assert_eq!(Locale::from_system("en"), Locale::new("en"));
    }

    /// 完整标签未命中时按主语言子标签精确匹配。
    #[test]
    fn from_system_language_match() {
        assert_eq!(Locale::from_system("zh-TW"), Locale::new("zh-CN"));
        assert_eq!(Locale::from_system("en-US"), Locale::new("en"));
        assert_eq!(Locale::from_system("EN_GB"), Locale::new("en"));
    }

    /// 系统语言与可用 locale 语言无交集时返回 None。
    #[test]
    fn from_system_no_match() {
        assert_eq!(Locale::from_system("de-DE"), None);
        assert_eq!(Locale::from_system(""), None);
        assert_eq!(Locale::from_system("."), None);
    }

    /// POSIX 形态归一化（下划线、编码、修饰后缀）。
    #[test]
    fn from_system_posix_normalization() {
        assert_eq!(Locale::from_system("zh_CN.UTF-8"), Locale::new("zh-CN"));
        assert_eq!(Locale::from_system("en_US@dict"), Locale::new("en"));
    }
}
