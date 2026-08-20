/**
 * 系统配置共享状态：模块级 $state 单例（ESM 仅加载一次），跨组件响应式共享。
 *
 * 真相源约定：config.json（后端）为系统级配置唯一真相源；本模块仅缓存
 * `get_config` 快照（经 `loadSystemConfig` 填充，单次 IPC 去重），
 * 写入侧（如 SystemSettings 的 toggle 命令成功回填）经 `getSystemConfig`
 * 取回同一代理对象原位赋值，消费方（WindowControl 托盘守卫、initLocale locale
 * 读取）经响应式派生联动。
 * Svelte 5 限制：$state 变量被重赋值时不可导出绑定（编译期报错），
 * 读经 `getSystemConfig()`（响应式上下文内调用仍跟踪）、写经 `setSystemConfig`。
 */

import type { SystemConfig } from "$libs/ipc";

/** 系统配置快照（null 表示尚未加载或加载失败）；原位属性赋值即触发消费方响应式更新 */
let systemConfig = $state<SystemConfig | null>(null);

/**
 * 读取共享配置快照：在响应式上下文（$derived/模板表达式）内调用仍跟踪变化。
 * @returns 当前配置快照（可能为 null）
 */
export function getSystemConfig(): SystemConfig | null {
  return systemConfig;
}

/**
 * 整体刷新共享状态（加载完成或需要整体替换时调用）。
 * Svelte 5 限制：跨模块不可整体重绑导入的 $state 变量，须经本函数刷新。
 * @param value 新的配置快照
 */
export function setSystemConfig(value: SystemConfig | null): void {
  systemConfig = value;
}
