/**
 * 打包功能类型契约（与后端 features/package.rs 对齐）
 */

export interface PackageFile {
  name: string;
  path: string;
  size: number;
}

export interface PackageIssue {
  id: string;
  severity: string;
  text: string;
  location?: string | null;
  position?: string | null;
  rule?: string | null;
}

export interface PackageResult {
  epubPath: string;
  txtPath: string;
  coverPath?: string | null;
  files: PackageFile[];
  bookTitle: string;
  verified: boolean;
  issues: PackageIssue[];
  epubVersion?: string | null;
}
