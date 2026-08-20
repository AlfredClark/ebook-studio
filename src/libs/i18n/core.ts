import { invokeCommand } from "$libs/ipc";
import { loadSystemConfig } from "$libs/system";
import { getLocale, setLocale, toLocale, type Locale } from "./paraglide/runtime";

/** 「跟随系统」模式哨兵值：与后端 cores/locale.rs 的 SYSTEM_LOCALE 镜像对齐；
 *  config.json 的 locale 存此值时语言跟随系统（运行期解析 + 回退 en） */
export const SYSTEM_LOCALE = "system" as const;

/** locale 存储模式值：跟随系统哨兵或具体语言标签 */
export type LocaleMode = Locale | typeof SYSTEM_LOCALE;

/**
 * 切换语言：先写后端 config.json 落盘，成功后才更新前端 paraglide 运行时。
 * 支持「跟随系统」模式值（"system"）；后端返回解析后的具体语言标签（跟随系统
 * 时返回系统解析结果，系统不可得回退 en），paraglide 运行时仅消费具体标签。
 * @param locale 目标语言：`system` 或具体语言标签
 * @param reload 是否刷新页面（默认 true；纯内存切换传 false 避免刷新循环）
 * @returns 后端写入与前端切换均成功时返回 true；后端写入失败时返回 false（前端不切换）
 */
export async function changeLocale(locale: LocaleMode, reload: boolean = true): Promise<boolean> {
  const result = await invokeCommand<Locale>("set_locale", { locale });
  if (result === null) {
    return false;
  }
  setLocale(result, { reload });
  return true;
}

/**
 * 启动时初始化 locale（onMount 调用）：
 * - 存储值为「跟随系统」模式：经 resolve_locale 解析具体语言标签（系统不可得回退 en）
 * - 前后端一致（正常情况）：无需任何修改——首帧已按 localStorage 渲染正确，仅同步 lang 属性
 * - 不一致（如外部修改 config.json，极少见）：以 config.json 为准，setLocale(reload:true)
 *   先写入 localStorage 再刷新——刷新后首帧即正确，此分支只执行一次
 * @returns 初始化成功时返回 true；命令失败或条目缺失时返回 false（保持 paraglide 默认语言）
 */
export async function initLocale(): Promise<boolean> {
  try {
    // 经共享缓存读取（与 SystemSettings/WindowControl 去重同一次 get_config IPC）
    const config = await loadSystemConfig();
    if (config === null) {
      return false;
    }
    // 跟随系统模式经 resolve_locale 解析；具体标签经 toLocale 校验（非法视为未命中）
    const resolved = config.locale === SYSTEM_LOCALE ? await invokeCommand<Locale>("resolve_locale") : toLocale(config.locale);
    if (resolved == null) {
      return false;
    }
    if (resolved !== getLocale()) {
      // 失同步自愈：写入 localStorage 并刷新，刷新后首帧即按新语言渲染
      // （reload 丢弃当前页，刷新后由首帧与后续 onMount 同步 lang 属性，无需在此赋值）
      setLocale(resolved, { reload: true });
      return true;
    }
    document.documentElement.lang = getLocale();
    return true;
  } catch {
    return false;
  }
}
