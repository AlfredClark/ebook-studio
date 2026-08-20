/**
 * 工作空间阶段定义：检查 > 拆分 > 清理 > 构建 > 打包
 */
import type { Pathname } from "$app/types";
import { m } from "$libs/i18n/paraglide/messages";

/** 阶段标识（与路由 slug 一致） */
export type WorkspaceStage = "inspect" | "split" | "clean" | "build" | "package";

/** 阶段元数据 */
export interface WorkspaceStageMeta {
  value: WorkspaceStage;
  label: () => string;
  /** 相对 workspace 根的 href 生成 */
  href: (id: string) => Pathname;
  desc: () => string;
}

export const WORKSPACE_STAGES: WorkspaceStageMeta[] = [
  {
    value: "inspect",
    label: m.workspace_stage_inspect,
    href: (id) => `/workspace/${id}/inspect` as Pathname,
    desc: m.workspace_stage_inspect_desc,
  },
  {
    value: "split",
    label: m.workspace_stage_split,
    href: (id) => `/workspace/${id}/split` as Pathname,
    desc: m.workspace_stage_split_desc,
  },
  {
    value: "clean",
    label: m.workspace_stage_clean,
    href: (id) => `/workspace/${id}/clean` as Pathname,
    desc: m.workspace_stage_clean_desc,
  },
  {
    value: "build",
    label: m.workspace_stage_build,
    href: (id) => `/workspace/${id}/build` as Pathname,
    desc: m.workspace_stage_build_desc,
  },
  {
    value: "package",
    label: m.workspace_stage_package,
    href: (id) => `/workspace/${id}/package` as Pathname,
    desc: m.workspace_stage_package_desc,
  },
];

const STAGE_SET = new Set<string>(WORKSPACE_STAGES.map((s) => s.value));

/**
 * 从 pathname 解析当前阶段（如 /workspace/<id>/inspect -> inspect）
 * 未匹配返回 undefined
 */
export function getActiveWorkspaceStage(pathname: string): WorkspaceStage | undefined {
  const parts = pathname.split("/").filter(Boolean);
  // ["workspace", "<id>", "<stage>", ...]
  if (parts.length < 3) return undefined;
  if (parts[0] !== "workspace") return undefined;
  const stage = parts[2];
  return STAGE_SET.has(stage) ? (stage as WorkspaceStage) : undefined;
}
