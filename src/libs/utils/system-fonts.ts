/**
 * 系统字体列表工具：从 tauri-plugin-system-fonts-api 获取系统已安装字体家族名数组（按家族名
 * 去重、清洗非法字符、本地化排序），带模块级缓存与 in-flight 去重。
 * 注意：插件返回的 `fontName` 是 post_script_name（如 "Inter-Regular"），CSS font-family
 * 须用 `name`（家族名，如 "Inter"）——本工具只消费 `name`。
 * 真相源约定：字体偏好（settings.font）真相源为 localStorage，应用侧经 stores/settings 的
 * 监听器写 --app-font-sans；本工具不持有响应式状态，仅提供一次性列表加载。
 */

import { getSystemFonts } from "tauri-plugin-system-fonts-api";
import { error as logError } from "$libs/logger";

/** 非法字符：CSS font-family 引号内无法安全承载的字符（引号/换行）。
 * 不带 g flag：test() 无 lastIndex 状态，避免连续调用时状态泄漏漏检（带 g 时 test 会推进 lastIndex） */
const INVALID_CHARS = /["\n\r]/;

/** in-flight 请求去重：同一时刻并发调用共享同一次插件调用（镜像 loadSystemConfig 模式） */
let inflight: Promise<string[]> | null = null;

/** 已加载缓存：加载完成后复用，避免重复 IPC（空数组也缓存，不再重试） */
let cached: string[] | null = null;

/**
 * 加载系统字体家族列表：已缓存直接返回；未缓存则调一次 `getSystemFonts`（并发去重）。
 * 返回的家族名已按 `name` 去重、剔除含非法字符的条目、按当前 locale 排序。
 * @returns 去重后的字体家族名数组；加载失败时返回空数组（调用方仅剩"系统默认"选项）
 */
export function loadSystemFonts(): Promise<string[]> {
  if (cached) return Promise.resolve(cached);
  if (!inflight) {
    inflight = getSystemFonts()
      .then((fonts) => {
        const names = new Set<string>();
        for (const font of fonts) {
          const name = font.name.trim();
          if (!name || INVALID_CHARS.test(name)) continue;
          names.add(name);
        }
        cached = [...names].sort(new Intl.Collator().compare);
        return cached;
      })
      .catch((cause) => {
        // 插件调用失败（非 Tauri 环境/无权限）：记日志，列表回退空数组，不阻断设置页
        logError(`[fonts] failed to load system fonts: ${cause instanceof Error ? cause.message : String(cause)}`).catch(
          () => {},
        );
        return [];
      })
      .finally(() => {
        inflight = null;
      });
  }
  return inflight;
}
