/**
 * 拆分业务：前端直调 invokeCommand 封装后端 split 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { SplitResult } from "./types";

/**
 * 查询已存在的拆分结果（若无则返回 null）
 * @param identifier 项目标识 urn:uuid:<uuid>
 */
export async function getSplitContent(identifier: string): Promise<SplitResult | null> {
  return invokeCommand<SplitResult>("get_split_content", { identifier });
}

/**
 * 执行拆分（解析 content.txt → 写 split.json）
 * @param identifier 项目标识 urn:uuid:<uuid>
 */
export async function splitContent(identifier: string): Promise<SplitResult | null> {
  return invokeCommand<SplitResult>("split_content", { identifier });
}

/**
 * 保存章节内容（落盘 split.json）
 * @param identifier 项目标识
 * @param volumeOrder 分卷编号（纯章节时为 null/undefined）
 * @param chapterOrder 章节编号
 * @param contents 行数组（已 trim 去空）
 */
export async function saveSplitChapter(
  identifier: string,
  volumeOrder: number | null | undefined,
  chapterOrder: number,
  contents: string[],
): Promise<SplitResult | null> {
  // 后端期望 volumeOrder: Option<i32>，驼峰转 volumeOrder
  return invokeCommand<SplitResult>("save_split_chapter", {
    identifier,
    volumeOrder: volumeOrder ?? null,
    chapterOrder,
    contents,
  });
}
