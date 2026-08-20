/**
 * 日志模块统一出口：重导出 tauri-plugin-log 运行时 API 与挂载工具。
 *
 * 与后端共用同一日志链路：Rust 侧经 log crate 宏写入，前端经本模块 API 写入，
 * 均落盘到应用日志目录（LogDir）。无自有类型契约，故省略 types.ts。
 */

export { trace, debug, info, warn, error, attachConsole } from "@tauri-apps/plugin-log";
export { initLogger } from "./core";
