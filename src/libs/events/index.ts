/**
 * 事件系统统一出口：事件契约（常量 + payload 类型）与监听封装。
 * 消费方一律经本模块使用，不直接 import @tauri-apps/api/event。
 */

export { listenEvent } from "./core";
export { EVENT_MENU_NAVIGATE, EVENT_REQUEST_EXIT, type MenuPage } from "./types";
