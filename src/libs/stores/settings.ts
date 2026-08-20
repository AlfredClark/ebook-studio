import { createStoreGroup, storeDef } from "./core";
import type { CloseBehaviorName, FontSizeName, FontWeightName, LayoutName, ThemeName } from "./types";
import { themeNames } from "$styles/themes";

/** 关闭行为合法值域（settings.closeBehavior 残留值校验用） */
const closeBehaviorNames: readonly CloseBehaviorName[] = ["ask", "quit", "minimize"];

/** 字体大小合法值域（settings.fontSize 残留值校验用；"" 默认档不在此列） */
const fontSizeNames: readonly FontSizeName[] = ["14", "16", "18", "20"];

/** 字体粗细合法值域（settings.fontWeight 残留值校验用；100-900 步进 100） */
const fontWeightNames: readonly FontWeightName[] = ["100", "200", "300", "400", "500", "600", "700", "800", "900"];

/**
 * UI 偏好统一出口：createStoreGroup 组合偏好子 store（layout/theme/closeBehavior/font/fontSize/fontWeight），
 * 各自独立持久化于 localStorage（key: layout/theme/close-behavior/font/font-size/font-weight），
 * 真相源与系统级配置（config.json）分离。
 * 暗色模式（.dark class）由 mode-watcher 负责（userPrefersMode 持久化于
 * mode-watcher-mode key，system 走 prefers-color-scheme），不在此维护。
 * 读写经成员访问：`$settings.layout` / `settings.theme.set(...)`。
 * 主题/字体（家族/大小/粗细）应用经 storeDef 的 subscribe 声明式注入：创建时应用当前值，此后每次变更跟随。
 */
export const settings = createStoreGroup({
  layout: storeDef<LayoutName>("default", "layout"),
  theme: storeDef<ThemeName>("neutral", "theme", createThemeListener()),
  closeBehavior: storeDef<CloseBehaviorName>("ask", "close-behavior"),
  checkUpdates: storeDef<boolean>(true, "check-updates"),
  font: storeDef<string>("", "font", createFontListener()),
  fontSize: storeDef<FontSizeName>("", "font-size", createFontSizeListener()),
  fontWeight: storeDef<FontWeightName>("400", "font-weight", createFontWeightListener()),
});

// 主题兜底：localStorage 残留已删除主题（如 red）时回退 neutral——
// 经 set 触发监听器（data-theme）与持久化同步修正，仅启动时执行一次
if (!themeNames.includes(settings.theme.get())) {
  settings.theme.set("neutral");
}

// 关闭行为兜底：残留非法值回退 ask（经 set 同步修正持久化，仅启动时执行一次）
if (!closeBehaviorNames.includes(settings.closeBehavior.get())) {
  settings.closeBehavior.set("ask");
}

// 字体大小兜底：残留非法值（如已移除的档位）回退 ""（经 set 同步修正持久化，仅启动时执行一次）
if (!fontSizeNames.includes(settings.fontSize.get())) {
  settings.fontSize.set("");
}

// 字体粗细兜底：残留非法值（如旧版 "" 或已移除的档位）回退 "400"（经 set 同步修正持久化，仅启动时执行一次）
if (!fontWeightNames.includes(settings.fontWeight.get())) {
  settings.fontWeight.set("400");
}

/**
 * 主题监听器工厂：将主题名写入 data-theme 属性（对应 themes/*.css 的 [data-theme="xxx"] 覆盖层；
 * neutral 同时以 :root 兜底首帧，显式设置后命中 [data-theme="neutral"] 规则）。
 * @returns 主题变更回调（经 storeDef subscribe 注入，创建时执行一次 + 每次变更触发）
 */
function createThemeListener(): (theme: ThemeName) => void {
  const root = document.documentElement;
  return (theme) => {
    root.setAttribute("data-theme", theme);
  };
}

/**
 * 字体监听器工厂：将字体家族名写入 --app-font-sans 自定义属性（app.css 的 --font-sans
 * 经 var() 引用，html 的 font-sans 全局生效）；空串恢复系统默认（删除属性回落默认栈）。
 * @returns 字体变更回调（经 storeDef subscribe 注入，创建时执行一次 + 每次变更触发）
 */
function createFontListener(): (font: string) => void {
  const root = document.documentElement;
  return (font) => {
    if (font) {
      // 字体名经引号包裹（含空格的家族名须作为整体解析）；非法字符已在 libs/utils/system-fonts 清洗
      root.style.setProperty("--app-font-sans", `"${font}"`);
    } else {
      root.style.removeProperty("--app-font-sans");
    }
  };
}

/**
 * 字体大小监听器工厂：将 px 档位写入 --app-font-size（app.css 的 html font-size 引用，
 * rem 基准全局等比缩放）；空串恢复默认（删除属性回落 16px）。
 * @returns 字体大小变更回调（经 storeDef subscribe 注入，创建时执行一次 + 每次变更触发）
 */
function createFontSizeListener(): (size: FontSizeName) => void {
  const root = document.documentElement;
  return (size) => {
    if (size) {
      root.style.setProperty("--app-font-size", `${size}px`);
    } else {
      root.style.removeProperty("--app-font-size");
    }
  };
}

/**
 * 字体粗细监听器工厂：将 font-weight 档位写入 --app-font-weight（app.css 的 html font-weight
 * 引用，全局默认字重；滑块必有值，恒写属性）。
 * @returns 字体粗细变更回调（经 storeDef subscribe 注入，创建时执行一次 + 每次变更触发）
 */
function createFontWeightListener(): (weight: FontWeightName) => void {
  const root = document.documentElement;
  return (weight) => {
    root.style.setProperty("--app-font-weight", weight);
  };
}
