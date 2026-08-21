/**
 * 清理业务：前端直调 invokeCommand 封装后端 clean 能力
 */
import { invokeCommand } from "$libs/ipc";
import type { CleanFilters, CleanResult } from "./types";

/**
 * 清理筛查（OR 语义，仅点击触发）
 * @param identifier 项目标识 urn:uuid:<uuid>
 * @param filters 勾选筛选项
 * @param customRegex 自定义正则（空即忽略）
 */
export async function filterClean(
  identifier: string,
  filters: CleanFilters,
  customRegex?: string,
): Promise<CleanResult | null> {
  const cr = customRegex?.trim() ? customRegex.trim() : undefined;
  return invokeCommand<CleanResult>("filter_clean", {
    identifier,
    filters,
    customRegex: cr ?? null,
  });
}
