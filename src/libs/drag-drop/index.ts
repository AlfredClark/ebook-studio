/**
 * 文件拖放能力统一出口：封装 Tauri 核心 API `onDragDropEvent`（窗口级文件拖放监听）。
 */

export { listenFileDrop } from "./core";
export type { DragDropEvent } from "@tauri-apps/api/window";
export type { Event, UnlistenFn } from "@tauri-apps/api/event";
