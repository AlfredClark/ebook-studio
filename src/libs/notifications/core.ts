/**
 * 系统通知能力层：封装 tauri-plugin-notification 的权限检查与发送。
 *
 * 官方插件自带 IPC 封装，不经 invokeCommand（与 dialog/fs/os 同模式）；
 * 权限 `notification:default` 已就绪（capabilities/plugins.json）。
 * 发送失败（无权限/系统不支持）仅记日志，不阻断调用方流程。
 * 复用 npm 包 API，无自有类型契约，故省略 types.ts。
 */

import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { error } from "$libs/logger";

/**
 * 发送系统通知：未授予权限时先请求（用户拒绝则放弃发送），发送失败仅记日志。
 * @param title 通知标题
 * @param body 通知正文
 * @returns 是否发送成功
 */
export async function notify(title: string, body: string): Promise<boolean> {
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      granted = (await requestPermission()) === "granted";
    }
    if (!granted) return false;
    sendNotification({ title, body });
    return true;
  } catch (err) {
    void error(`[notification] send failed: ${err instanceof Error ? err.message : String(err)}`).catch(() => {});
    return false;
  }
}
