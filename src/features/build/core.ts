/**
 * 构建业务：前端直调 invokeCommand 封装后端 build 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { BuildResult } from "./types";

/**
 * 查询已构建的 EPUB 目录（若不存在返回 null）
 * @param identifier 项目标识 urn:uuid:<uuid>
 */
export async function getBuild(identifier: string): Promise<BuildResult | null> {
  return invokeCommand<BuildResult>("get_build", { identifier });
}

/**
 * 执行构建（基于 split.json + metadata.json 生成未压缩 EPUB）
 * @param identifier 项目标识
 */
export async function buildEpub(identifier: string): Promise<BuildResult | null> {
  return invokeCommand<BuildResult>("build_epub", { identifier });
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
