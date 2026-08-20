/**
 * 检查功能类型契约（与后端 features/inspect.rs 对齐）
 */

export type InspectStructure = "volume_chapters" | "pure_chapters" | "volume_intro" | "auto";

export interface VolumeInfo {
  index: number;
  num: number | null;
  rawNum?: string | null;
  title: string;
  lineNo: number;
  hasIntro: boolean;
  intro?: string | null;
  chapterCount: number;
}

export interface ChapterInfo {
  index: number;
  num: number | null;
  rawNum: string;
  title: string;
  lineNo: number;
  volumeIndex: number | null;
  volumeTitle?: string | null;
}

export interface InspectIssue {
  lineNo: number | null;
  kind: string;
  message: string;
  expected?: string | null;
  actual?: string | null;
  context?: string | null;
}

export interface InspectStats {
  totalLines: number;
  totalVolumes: number;
  totalChapters: number;
}

export interface InspectResult {
  detectedStructure: string;
  requestedStructure: string;
  volumes: VolumeInfo[];
  chapters: ChapterInfo[];
  issues: InspectIssue[];
  stats: InspectStats;
  absPath?: string | null;
}
