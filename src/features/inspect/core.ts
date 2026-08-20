/**
 * 检查业务：前端直调 invokeCommand 封装后端 inspect 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { InspectResult } from "./types";

/**
 * 检查 content 结构
 * @param identifier 项目标识 urn:uuid:<uuid>
 * @param structure 结构提示 auto | volume_chapters | pure_chapters | volume_intro
 * @param volumeRegex 自定义分卷正则（留空用默认）
 * @param chapterRegex 自定义分章正则（留空用默认）
 */
export async function inspectContent(
  identifier: string,
  structure: string = "auto",
  volumeRegex?: string,
  chapterRegex?: string,
): Promise<InspectResult | null> {
  const v = volumeRegex?.trim() ? volumeRegex.trim() : undefined;
  const c = chapterRegex?.trim() ? chapterRegex.trim() : undefined;
  return invokeCommand<InspectResult>("inspect_content", {
    identifier,
    structure,
    volumeRegex: v,
    chapterRegex: c,
  });
}

/**
 * 重整章节与卷编号（保留标题）
 */
export async function reorderChapters(
  identifier: string,
  structure: string = "auto",
  volumeRegex?: string,
  chapterRegex?: string,
): Promise<InspectResult | null> {
  const v = volumeRegex?.trim() ? volumeRegex.trim() : undefined;
  const c = chapterRegex?.trim() ? chapterRegex.trim() : undefined;
  return invokeCommand<InspectResult>("reorder_chapters", {
    identifier,
    structure,
    volumeRegex: v,
    chapterRegex: c,
  });
}

/**
 * 获取 content 绝对路径（供 opener 打开）
 */
export async function getContentPath(identifier: string): Promise<string | null> {
  return invokeCommand<string>("get_content_path", { identifier });
}
