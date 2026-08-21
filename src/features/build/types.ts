/**
 * 构建功能类型契约（与后端 features/build.rs 对齐）
 */

export interface BuildFile {
  path: string;
  name: string;
  isDir: boolean;
  children?: BuildFile[] | null;
}

export interface BuildResult {
  epubPath: string;
  files: BuildFile[];
  bookTitle: string;
}
