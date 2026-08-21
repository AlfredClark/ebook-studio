/**
 * 拆分功能类型契约（与后端 features/split.rs 对齐）
 */

/** 章节 */
export interface SplitChapter {
  order: number;
  title: string;
  contents: string[];
}

/** 分卷（含 intro 与章节） */
export interface SplitVolume {
  order: number;
  title: string;
  intro?: string[] | null;
  chapters: SplitChapter[];
}

/** 拆分结果：
 * - pure_chapters: { type, chapters }
 * - volume_*: { type, volumes }
 */
export type SplitResult =
  | { type: "pure_chapters"; chapters: SplitChapter[]; volumes?: never }
  | { type: "volume_chapters"; volumes: SplitVolume[]; chapters?: never }
  | { type: "volume_intro"; volumes: SplitVolume[]; chapters?: never };

export type SplitType = SplitResult["type"];
