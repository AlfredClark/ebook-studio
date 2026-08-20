/**
 * 清洗业务类型契约：与后端 `features/clean.rs` 的 TxtInfo/TxtDetail 对齐。
 */

/** 列表项 */
export interface TxtInfo {
  name: string;
  path: string;
  size: number;
  mtime: number;
}

/** 详情：含行数与字符数 */
export interface TxtDetail extends TxtInfo {
  lines: number;
  chars: number;
}

/** 排序键 */
export type SortKey = "nameAsc" | "nameDesc" | "mtimeDesc" | "mtimeAsc" | "sizeDesc" | "sizeAsc";
