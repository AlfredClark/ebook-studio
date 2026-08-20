/**
 * 项目业务：前端直调 invokeCommand 封装后端 Projects 能力。
 */

import { invokeCommand } from "$libs/ipc";
import type { CreateProjectInput, Project } from "./types";

/**
 * 列出项目（后端扫描 Projects 目录）
 * @returns 项目列表，失败返回 null
 */
export async function listProjects(): Promise<Project[] | null> {
  return invokeCommand<Project[]>("list_projects");
}

/**
 * 创建项目
 * @param input 创建输入（含原始文件路径 coverSrc/contentSrc）
 * @returns 创建的项目，失败返回 null
 */
export async function createProject(input: CreateProjectInput): Promise<Project | null> {
  return invokeCommand<Project>("create_project", { payload: input });
}

/**
 * 获取单个项目详情
 * @param identifier 项目标识
 * @returns 项目详情，失败返回 null
 */
export async function getProject(identifier: string): Promise<Project | null> {
  return invokeCommand<Project>("get_project", { identifier });
}

/**
 * 删除单个项目（整目录）
 * @param identifier 项目标识
 * @returns 成功返回 true，失败返回 null
 */
export async function deleteProject(identifier: string): Promise<boolean | null> {
  return invokeCommand<boolean>("delete_project", { identifier });
}

/**
 * 批量删除项目
 * @param identifiers 项目标识数组
 * @returns 成功删除数，失败返回 null
 */
export async function batchDeleteProjects(identifiers: string[]): Promise<number | null> {
  return invokeCommand<number>("batch_delete_projects", { identifiers });
}

/**
 * 解析项目资产为可预览的绝对路径
 * @param identifier 项目标识
 * @param relative 相对路径（如 sources/cover.jpg）
 * @returns 绝对路径，失败返回 null
 */
export async function resolveProjectAsset(identifier: string, relative: string): Promise<string | null> {
  return invokeCommand<string>("resolve_project_asset", { identifier, relative });
}
