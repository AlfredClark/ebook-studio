/**
 * 清洗业务：前端直调 `invokeCommand` 封装后端 txts 目录能力。
 */

import { invokeCommand } from "$libs/ipc";
import type { TxtDetail, TxtInfo } from "./types";

/**
 * 列出 txts 文件
 * @returns 文件列表，失败返回 null
 */
export async function listTxts(): Promise<TxtInfo[] | null> {
  return invokeCommand<TxtInfo[]>("list_txts");
}

/**
 * 复制文件/目录到 txts（递归收集 txt）
 * @param srcPath 源绝对路径（文件或目录）
 * @returns 复制后的文件列表，失败返回 null，未找到 txt 返回空数组
 */
export async function copyTxt(srcPath: string): Promise<TxtInfo[] | null> {
  return invokeCommand<TxtInfo[]>("copy_txt", { srcPath });
}

/**
 * 获取详情（含行数/字符数）
 * @param name 文件名
 * @returns 详情，失败返回 null
 */
export async function getTxtDetail(name: string): Promise<TxtDetail | null> {
  return invokeCommand<TxtDetail>("get_txt_detail", { name });
}

/**
 * 删除文件
 * @param name 文件名
 * @returns 成功返回 true，失败返回 null
 */
export async function deleteTxt(name: string): Promise<boolean | null> {
  return invokeCommand<boolean>("delete_txt", { name });
}
