/**
 * 事件监听封装：统一出口包装 `@tauri-apps/api/event` 的 listen，
 * 消费方不直接依赖官方 API（与 ipc/logger 同模式）；handler 接收解包后的 payload。
 */

import { listen } from "@tauri-apps/api/event";

/**
 * 注册事件监听（异步注册，resolve 后返回 unlisten 函数）。
 * 调用方负责生命周期：组件销毁时调用返回的 unlisten（listen 异步 resolve
 * 晚于销毁时须先判定 disposed 再决定清理或留存，见 (main)/+layout.svelte 示例）。
 * @param event 事件名（一律使用 types.ts 的事件常量，不写字符串字面量）
 * @param handler 事件处理函数（参数为解包后的 payload）
 * @returns resolve 后返回的 unlisten 函数（取消监听）
 */
export async function listenEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  return listen<T>(event, (e) => handler(e.payload));
}
