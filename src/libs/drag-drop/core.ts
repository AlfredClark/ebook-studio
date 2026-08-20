/**
 * 文件拖放能力层：封装 Tauri 核心 API `onDragDropEvent`（免插件免权限，
 * 窗口级监听——窗口任意位置拖入文件均触发）。
 *
 * 事件为判别联合：`enter`（含 paths）/ `over`（悬停）/ `drop`（含 paths）/ `leave`；
 * 调用方按需消费（如 demo 页以 enter/over 驱动高亮、drop 取路径）。
 * 生命周期同 listenEvent 约定：异步 resolve，调用方管理 unlisten 与 disposed 守卫。
 */

import { getCurrentWindow, type DragDropEvent } from "@tauri-apps/api/window";
import type { Event, UnlistenFn } from "@tauri-apps/api/event";

/**
 * 监听文件拖放事件（窗口级）。
 * @param handler 拖放事件回调（收 `Event<DragDropEvent>`，payload 见 DragDropEvent 判别联合）
 * @returns unlisten 函数（组件销毁时调用）
 */
export function listenFileDrop(handler: (event: Event<DragDropEvent>) => void): Promise<UnlistenFn> {
  return getCurrentWindow().onDragDropEvent(handler);
}
