/**
 * 系统配置模块统一出口：共享响应式状态（state.svelte.ts 单例）与业务函数
 * （快照加载、toggle 系列、置顶能力探测）。
 * 无自有类型契约（复用 ipc/types.ts 的 SystemConfig），省略 types.ts。
 */

export { getSystemConfig, setSystemConfig } from "./state.svelte";
export {
  isAlwaysOnTopSupported,
  loadSystemConfig,
  resetSystemConfig,
  toggleAutostart,
  toggleNotification,
  toggleTray,
  toggleWindowState,
} from "./core";
