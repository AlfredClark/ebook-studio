/**
 * 清理功能类型契约（与后端 features/clean.rs 对齐）
 */

export interface CleanFilters {
  halfPunct: boolean;
  specialSymbol: boolean;
  unclosedPairs: boolean;
  repeatedPunct: boolean;
  abnormalWhitespace: boolean;
  ellipsisDash: boolean;
}

export interface CleanIssue {
  volumeOrder: number | null;
  chapterOrder: number;
  lineIndex: number;
  kind: string;
  message: string;
  context: string;
  matched?: string | null;
}

export interface CleanMatched {
  volumeOrder: number | null;
  chapterOrder: number;
}

export interface CleanResult {
  matched: CleanMatched[];
  issues: CleanIssue[];
}

export type CleanFilterKey =
  | "half_punct"
  | "special_symbol"
  | "unclosed_pairs"
  | "repeated_punct"
  | "abnormal_whitespace"
  | "ellipsis_dash"
  | "custom_regex";
