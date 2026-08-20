import { attachConsole } from "@tauri-apps/plugin-log";

/**
 * 挂载日志控制台镜像：将插件日志（Rust + 前端）转发到浏览器控制台。
 * 需后端启用 Webview target（见 cores/logger.rs）；应用启动时调用一次。
 * @returns 无返回值；调用失败时静默忽略（日志功能退化不影响应用）
 */
export async function initLogger(): Promise<void> {
  try {
    await attachConsole();
  } catch {
    // 非 Tauri 环境（纯前端 dev）或无权限时忽略
  }
}
