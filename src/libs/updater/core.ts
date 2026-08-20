/**
 * 应用自动更新：封装 tauri-plugin-updater 的检查、下载安装与重启流程。
 *
 * 使用方式：
 * - `checkForUpdate()` 检查是否有新版本可用（无新版本返回 null）
 * - `installUpdate(update, onProgress)` 下载安装并通过回调报告进度（不重启）
 * - `relaunchApp()` 重启应用（安装完成后手动触发，失败不视为更新失败）
 */

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "$libs/process";

/**
 * 检查是否有新版本可用。
 * @returns 有新版本时返回 Update 对象，否则返回 null；配置缺失（pubkey/endpoints 未配置）时抛错
 */
export async function checkForUpdate(): Promise<Update | null> {
  return check();
}

/**
 * 下载并安装更新（不重启；安装完成后经 `relaunchApp` 重启）。
 * @param update checkForUpdate 返回的 Update 对象
 * @param onProgress 下载进度回调，参数为（已下载字节数, 总字节数；总字节数在 Started 事件后确定）
 */
export async function installUpdate(update: Update, onProgress: (downloaded: number, total?: number) => void): Promise<void> {
  let downloaded = 0;
  let contentLength: number | undefined;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        contentLength = event.data.contentLength;
        onProgress(0, contentLength);
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        onProgress(downloaded, contentLength);
        break;
      case "Finished":
        break;
    }
  });
}

/**
 * 重启应用（安装完成后调用；重启失败不影响已安装的更新，调用方仅记日志）。
 */
export async function relaunchApp(): Promise<void> {
  await relaunch();
}
