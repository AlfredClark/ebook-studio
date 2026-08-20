/**
 * 项目类型契约（与后端 `features/projects.rs` 的 ProjectMetadata 对齐，Dublin Core）。
 */

/** 项目信息（完整 Dublin 字段） */
export interface Project {
  /** 标识符 `urn:uuid:<uuid>` */
  identifier: string;
  /** 标题（必填） */
  title: string;
  /** 语言标签（必填，en | zh-CN） */
  language: string;
  /** 创作者（作者） */
  creator?: string | null;
  contributor?: string | null;
  publisher?: string | null;
  /** 日期 YYYY-MM-DD */
  date?: string | null;
  /** 主题数组（"/" 分割后） */
  subjects: string[];
  /** 简介数组（每行一个元素） */
  description: string[];
  rights?: string | null;
  source?: string | null;
  relation?: string | null;
  coverage?: string | null;
  /** 创建时间 RFC3339 */
  created: string;
  /** 修改时间 RFC3339 */
  modified: string;
  /** 修改时间戳毫秒（排序键） */
  modifiedMs: number;
  /** 封面相对路径 `sources/cover.<ext>` */
  cover?: string | null;
  /** 正文相对路径 `sources/content.txt` */
  content?: string | null;
}

/** 创建项目输入（前端表单提交） */
export interface CreateProjectInput {
  title: string;
  language: string;
  creator?: string;
  contributor?: string;
  publisher?: string;
  date?: string;
  /** 原始 subjects 字符串（"/" 分割），后端拆数组 */
  subjects?: string;
  description?: string;
  rights?: string;
  source?: string;
  relation?: string;
  coverage?: string;
  /** 封面原始绝对路径 */
  coverSrc?: string | null;
  /** 正文原始绝对路径 */
  contentSrc?: string | null;
}

/** 更新项目输入（与 CreateProjectInput 同构，增加 identifier 与移除标记） */
export interface UpdateProjectInput extends CreateProjectInput {
  /** 标识符 `urn:uuid:<uuid>` */
  identifier: string;
  /** 是否移除现有封面（coverSrc 为空时生效） */
  removeCover?: boolean;
  /** 是否移除现有正文 */
  removeContent?: boolean;
}

/** 排序键（默认最新修改优先） */
export type ProjectSortKey = "modifiedDesc" | "modifiedAsc";
