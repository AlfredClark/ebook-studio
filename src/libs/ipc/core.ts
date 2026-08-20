import { invoke } from "@tauri-apps/api/core";
import { error } from "$libs/logger";
import type { Response } from "./types";
import { CODE_OK } from "./types";

/**
 * 调用 Rust 命令并解包统一响应（Response<T>）。
 * 业务失败（code !== 0）或调用异常（命令未注册、非 Tauri 环境等）时
 * 写入共享日志链路并返回 null；成功后返回响应中的 data。
 * @param command Rust 命令名（invoke 第一个参数）
 * @param args 命令参数对象，键名与 Rust 参数名一致（Tauri 自动转换驼峰命名）
 * @returns 业务数据；失败时返回 null
 */
export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const response = await invoke<Response<T>>(command, args);
    if (response.code !== CODE_OK) {
      void error(`[ipc] ${command} 失败 (code=${response.code}): ${response.message}`).catch(() => {});
      return null;
    }
    return response.data;
  } catch (err) {
    void error(`[ipc] ${command} 调用异常: ${err}`).catch(() => {});
    return null;
  }
}
