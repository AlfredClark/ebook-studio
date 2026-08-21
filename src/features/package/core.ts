/**
 * 打包业务：前端直调 invokeCommand 封装后端 package 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { PackageResult } from "./types";

/**
 * 查询已打包的产物（若无则 null）
 * @param identifier 项目标识 urn:uuid:<uuid>
 */
export async function getPackage(identifier: string): Promise<PackageResult | null> {
  return invokeCommand<PackageResult>("get_package", { identifier });
}

/**
 * 执行打包（压缩 build 目录为 epub + 生成 txt + 拷贝封面 + 校验）
 * @param identifier 项目标识
 */
export async function packageEpub(identifier: string): Promise<PackageResult | null> {
  return invokeCommand<PackageResult>("package_epub", { identifier });
}

/**
 * 校验已打包的 EPUB（按需触发）
 * @param identifier 项目标识
 */
export async function verifyPackage(identifier: string): Promise<PackageResult | null> {
  return invokeCommand<PackageResult>("verify_package", { identifier });
}

/**
 * 删除 outputs 目录
 * @param identifier 项目标识
 */
export async function removePackage(identifier: string): Promise<boolean> {
  const r = await invokeCommand<boolean>("remove_package", { identifier });
  return r ?? false;
}

/**
 * 获取 outputs 目录绝对路径（供 opener 揭示）
 */
export async function getPackagePath(identifier: string): Promise<string | null> {
  return invokeCommand<string>("get_package_path", { identifier });
}
