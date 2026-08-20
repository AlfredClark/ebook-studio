/**
 * 更新模块统一出口：封装 tauri-plugin-updater 的检查、下载安装与重启流程，
 * 并提供跨组件共享的更新流程状态（state.svelte.ts 模块级单例）。
 *
 * 复用 npm 包类型（Update），无自有类型契约，故省略 types.ts。
 */

export { checkForUpdate, installUpdate } from "./core";
export { checkUpdate, checkUpdateSilently, installPendingUpdate, relaunchApp, update } from "./state.svelte";
export type { UpdateState } from "./state.svelte";
export type { Update } from "@tauri-apps/plugin-updater";
