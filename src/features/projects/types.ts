/**
 * 项目类型契约（EPUB3.3 必选四字段）。
 * 后续接入数据库时与后端保持一致。
 */

/** 项目信息 */
export interface Project {
  /** 标识符（EPUB identifier，唯一主键） */
  identifier: string;
  /** 标题 */
  title: string;
  /** 语言标签（BCP47，如 en / zh-CN / ja） */
  language: string;
  /** 最后修改时间（epoch ms，对应 EPUB dcterms:modified） */
  modified: number;
}

/** 排序键（默认最新修改优先） */
export type ProjectSortKey = "modifiedDesc" | "modifiedAsc";
