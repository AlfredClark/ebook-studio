/**
 * 更新流程全局状态：模块级 $state 单例（ESM 仅加载一次），跨组件共享。
 *
 * 状态变更集中在动作内（checkUpdate / installPendingUpdate / relaunchApp），组件只读渲染。
 * 变更经 $state 代理属性赋值（update.status = ...）而非整体重赋值，
 * 保证模块级状态的读取跟踪可靠。
 *
 * 安装与重启分离：安装成功即置 `installed`（重启失败不误报更新失败）；
 * 重启由用户经 relaunchApp 触发（安装完成后更新已生效，重启仅使其在运行进程上生效）。
 */

import { error } from "$libs/logger";
import type { Update } from "@tauri-apps/plugin-updater";
import { checkForUpdate, installUpdate, relaunchApp as doRelaunch } from "./core";

/** 更新流程状态：idle 可检查 → checking 检查中 → available 有新版本 → downloading 下载安装中 →
 *  installed 已安装待重启 → upToDate/error 为内联反馈态 */
export type UpdateState = {
  status: "idle" | "checking" | "available" | "downloading" | "installed" | "upToDate" | "error";
  /** 新版本号（status 为 available 时有值） */
  version?: string;
  /** 下载进度百分比（status 为 downloading 时回填；total 未确定时为空） */
  percent?: number;
};

/** 更新流程状态（模块级单例，仅加载一次） */
export const update = $state<UpdateState>({ status: "idle" });

/** 已检出的更新对象（非渲染数据，普通模块变量即可；installPendingUpdate 前经 checkUpdate 赋值） */
let pendingUpdate: Update | null = null;

/** 检查更新：无新版本 → upToDate（可再次检查）；有新版本 → available 等待确认；失败 → error + 日志 */
export async function checkUpdate(): Promise<void> {
  update.status = "checking";
  try {
    const available = await checkForUpdate();
    if (!available) {
      update.status = "upToDate";
      return;
    }
    pendingUpdate = available;
    update.status = "available";
    update.version = available.version;
  } catch (err) {
    await error(`[updater] check update failed: ${err instanceof Error ? err.message : String(err)}`).catch(() => {});
    update.status = "error";
  }
}

/**
 * 静默检查更新（启动期自动检查用）：与 checkUpdate 的区别——
 * 失败仅记日志、不触碰共享状态（避免关于页误显错误态）；有更新时
 * 回填共享状态（关于页直接显示安装按钮）并返回更新对象供调用方提示。
 * @returns 有新版本时返回 Update 对象；无新版本或检查失败时返回 null
 */
export async function checkUpdateSilently(): Promise<Update | null> {
  try {
    const available = await checkForUpdate();
    if (!available) return null;
    pendingUpdate = available;
    update.status = "available";
    update.version = available.version;
    return available;
  } catch (err) {
    await error(`[updater] silent check update failed: ${err instanceof Error ? err.message : String(err)}`).catch(() => {});
    return null;
  }
}

/** 下载安装已检出的更新：进度回填 percent（total 在 Started 事件后确定）；
 *  安装成功 → installed（待用户重启）；安装失败 → error */
export async function installPendingUpdate(): Promise<void> {
  if (update.status !== "available" || !pendingUpdate) return;
  update.status = "downloading";
  try {
    await installUpdate(pendingUpdate, (downloaded, total) => {
      update.status = "downloading";
      update.percent = total ? Math.round((downloaded / total) * 100) : undefined;
    });
    update.status = "installed";
  } catch (err) {
    await error(`[updater] install update failed: ${err instanceof Error ? err.message : String(err)}`).catch(() => {});
    update.status = "error";
  }
}

/** 重启应用：安装完成后触发；重启失败仅记日志（更新已安装，不污染更新状态） */
export async function relaunchApp(): Promise<void> {
  try {
    await doRelaunch();
  } catch (err) {
    await error(`[updater] relaunch failed: ${err instanceof Error ? err.message : String(err)}`);
  }
}
