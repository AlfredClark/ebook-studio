/**
 * 系统配置业务：快照加载/缓存（in-flight 去重，并发调用只发一次 IPC）、
 * toggle 系列编排（调命令 → 成功后回填共享状态 → 返回新值）、置顶能力探测。
 * 真相源约定：config.json（后端）为系统级配置唯一真相源；共享状态（state.svelte.ts）
 * 为本模块缓存快照，消费方（WindowControl 托盘守卫、initLocale locale 读取）响应式联动。
 * 跨领域联动（如托盘关闭 → closeBehavior 回退）属 UI 层协调，留在组件内，不归本模块。
 */

import { invokeCommand, type SystemConfig } from "$libs/ipc";
import { getSystemConfig, setSystemConfig } from "./state.svelte";

/** in-flight 请求去重：同一时刻并发调用共享同一次 IPC */
let inflight: Promise<SystemConfig | null> | null = null;

/**
 * 加载系统配置快照：已缓存直接返回；未缓存则发起一次 `get_config`（并发去重）。
 * @returns 配置快照；加载失败时返回 null（缓存保持 null，下次调用重试）
 */
export function loadSystemConfig(): Promise<SystemConfig | null> {
  const cached = getSystemConfig();
  if (cached) return Promise.resolve(cached);
  if (!inflight) {
    inflight = invokeCommand<SystemConfig>("get_config")
      .then((value) => {
        // 整体赋值触发消费方响应式更新（经 setter 规避跨模块 $state 重绑限制）
        setSystemConfig(value);
        return value;
      })
      .finally(() => {
        inflight = null;
      });
  }
  return inflight;
}

/**
 * 切换开机自启：命令成功（先 OS 生效再落盘）后回填共享状态。
 * @returns 切换后的 autostart 值；失败时返回 null（共享状态保持原值）
 */
export async function toggleAutostart(): Promise<boolean | null> {
  const next = await invokeCommand<boolean>("toggle_autostart");
  const config = getSystemConfig();
  if (next !== null && config) config.autostart = next;
  return next;
}

/**
 * 切换系统托盘：命令成功（先 OS 生效再落盘）后回填共享状态。
 * 关闭托盘导致 minimize 关闭行为失效的联动由调用方（UI 层）处理，本函数只返回新值。
 * @returns 切换后的 tray 值；失败时返回 null（共享状态保持原值）
 */
export async function toggleTray(): Promise<boolean | null> {
  const next = await invokeCommand<boolean>("toggle_tray");
  const config = getSystemConfig();
  if (next !== null && config) config.tray = next;
  return next;
}

/**
 * 切换系统通知：命令成功后回填共享状态。
 * @returns 切换后的 notification 值；失败时返回 null（共享状态保持原值）
 */
export async function toggleNotification(): Promise<boolean | null> {
  const next = await invokeCommand<boolean>("toggle_notification");
  const config = getSystemConfig();
  if (next !== null && config) config.notification = next;
  return next;
}

/**
 * 切换窗口状态记忆：命令成功后回填共享状态。
 * @returns 切换后的 window_state 值；失败时返回 null（共享状态保持原值）
 */
export async function toggleWindowState(): Promise<boolean | null> {
  const next = await invokeCommand<boolean>("toggle_window_state");
  const config = getSystemConfig();
  if (next !== null && config) config.windowState = next;
  return next;
}

/**
 * 查询窗口置顶能力：Linux Wayland 下 GTK keep_above 无效，前端据此隐藏置顶按钮。
 * 查询失败默认按支持处理（`?? true` 兜底，不因探测失败隐藏可用能力）。
 * @returns 是否支持窗口置顶
 */
export async function isAlwaysOnTopSupported(): Promise<boolean> {
  const supported = await invokeCommand<boolean>("is_always_on_top_supported");
  return supported ?? true;
}

/**
 * 恢复全部系统配置为默认值（命令内先 OS 生效再落盘，locale 一并重置为系统语言）。
 * 成功后整体回填共享状态；UI 偏好（localStorage）的重置由调用方（组件层）编排。
 * @returns 重置后的配置快照；失败时返回 null（共享状态保持原值）
 */
export async function resetSystemConfig(): Promise<SystemConfig | null> {
  const next = await invokeCommand<SystemConfig>("reset_config");
  if (next) setSystemConfig(next);
  return next;
}
