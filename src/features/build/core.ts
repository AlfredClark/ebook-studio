/**
 * 构建业务：前端直调 invokeCommand 封装后端 build 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { BuildResult, FormatConfig } from "./types";

/**
 * 查询已构建的 EPUB 目录（若不存在返回 null）
 * @param identifier 项目标识 urn:uuid:<uuid>
 */
export async function getBuild(identifier: string): Promise<BuildResult | null> {
  return invokeCommand<BuildResult>("get_build", { identifier });
}

/**
 * 执行构建（基于 split.json + metadata.json 生成未压缩 EPUB，支持标题与编号格式）
 * @param identifier 项目标识
 * @param chapterTitleFormat 章节标题格式，如 第{order}章 {title}
 * @param volumeTitleFormat 卷标题格式，如 第{order}卷 {title}
 * @param numberFormat 编号格式：arabic | arabic_padded | chinese_lower | chinese_upper
 */
export async function buildEpub(
  identifier: string,
  chapterTitleFormat?: string,
  volumeTitleFormat?: string,
  numberFormat?: string,
): Promise<BuildResult | null> {
  return invokeCommand<BuildResult>("build_epub", {
    identifier,
    chapterTitleFormat: chapterTitleFormat?.trim() || null,
    volumeTitleFormat: volumeTitleFormat?.trim() || null,
    numberFormat: numberFormat?.trim() || null,
  });
}

/**
 * 删除构建目录（重新构建前）
 * @param identifier 项目标识
 */
export async function removeBuild(identifier: string): Promise<boolean> {
  const r = await invokeCommand<boolean>("remove_build", { identifier });
  return r ?? false;
}

/**
 * 读取构建目录下指定文件的文本内容（仅文本文件）
 * @param identifier 项目标识
 * @param relPath 相对构建根的路径（如 EPUB/content.opf）
 */
export async function readBuildFile(identifier: string, relPath: string): Promise<string | null> {
  return invokeCommand<string>("read_build_file", { identifier, relPath });
}

/**
 * 写入构建目录下指定文件的文本内容
 * @param identifier 项目标识
 * @param relPath 相对路径
 * @param content 文本内容
 */
export async function writeBuildFile(identifier: string, relPath: string, content: string): Promise<boolean> {
  const r = await invokeCommand<boolean>("write_build_file", { identifier, relPath, content });
  return r ?? false;
}

/**
 * 获取构建目录绝对路径（供 opener 揭示）
 */
export async function getBuildPath(identifier: string): Promise<string | null> {
  return invokeCommand<string>("get_build_path", { identifier });
}

/**
 * 获取已保存的格式化配置（format.json）
 */
export async function getFormat(identifier: string): Promise<FormatConfig | null> {
  return invokeCommand<FormatConfig>("get_format", { identifier });
}
