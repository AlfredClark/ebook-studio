/**
 * 进程能力统一出口：重导出 tauri-plugin-process 的 exit / relaunch。
 *
 * 错误约定：薄层透传不吞错，失败由调用方处理（如 updater 的 relaunchApp 捕获后记日志）。
 * 用法示例：`import { relaunch } from "$libs/process"`。
 */

export { exit, relaunch } from "@tauri-apps/plugin-process";
