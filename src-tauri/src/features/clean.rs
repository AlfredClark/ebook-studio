//! 清理功能：基于 split.json 的行级筛查
//!
//! 真相源：`APPDATA/Projects/<uuid>/split.json`（由 split 阶段生成）
//! 筛选项 OR + 正则 OR：任意勾选命中或正则命中则该行命中，任意行命中则该章命中

#![allow(clippy::collapsible_if)]

use std::{collections::HashSet, fs, path::PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::cores::response::{AppError, AppResult, CODE_ERROR};
use crate::features::split::SplitResult;

const PROJECTS_DIR: &str = "Projects";
const SPLIT_NAME: &str = "split.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanFilters {
    #[serde(default)]
    pub half_punct: bool,
    #[serde(default)]
    pub special_symbol: bool,
    #[serde(default)]
    pub unclosed_pairs: bool,
    #[serde(default)]
    pub repeated_punct: bool,
    #[serde(default)]
    pub abnormal_whitespace: bool,
    #[serde(default)]
    pub ellipsis_dash: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanIssue {
    pub volume_order: Option<i32>,
    pub chapter_order: i32,
    /// 0-based 行索引（contents 内），展示时 +1
    pub line_index: usize,
    pub kind: String,
    pub message: String,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanMatched {
    pub volume_order: Option<i32>,
    pub chapter_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub matched: Vec<CleanMatched>,
    pub issues: Vec<CleanIssue>,
}

fn sanitize_identifier(identifier: &str) -> AppResult<String> {
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    uuid::Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[clean] 非法 identifier"))?;
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[clean] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn resolve_split_path(app: &AppHandle, identifier: &str) -> AppResult<PathBuf> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[clean] resolve Projects dir 失败: {e}")))?;
    Ok(base.join(&uuid).join(SPLIT_NAME))
}

fn is_unclosed(line: &str) -> bool {
    // 成对标点 distinct open/close
    let pairs: &[(char, char)] = &[
        ('（', '）'),
        ('(', ')'),
        ('【', '】'),
        ('[', ']'),
        ('《', '》'),
        ('“', '”'),
        ('‘', '’'),
    ];
    for (open, close) in pairs {
        let o = line.chars().filter(|c| c == open).count();
        let c = line.chars().filter(|c| c == close).count();
        if o != c {
            return true;
        }
    }
    // 同字符成对： "  '  ` 需偶数
    for ch in ['"', '\'', '`'] {
        let cnt = line.chars().filter(|c| *c == ch).count();
        if cnt % 2 != 0 {
            return true;
        }
    }
    // 全角引号已在 pairs 中，额外检查：若只出现一侧也算未闭合已由 count 覆盖
    false
}

/// 清理筛查（OR 语义）
pub(crate) fn filter_clean(
    app: &AppHandle,
    identifier: &str,
    filters: CleanFilters,
    custom_regex: Option<String>,
) -> AppResult<CleanResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[clean] identifier 不能为空"));
    }

    let any_filter = filters.half_punct
        || filters.special_symbol
        || filters.unclosed_pairs
        || filters.repeated_punct
        || filters.abnormal_whitespace
        || filters.ellipsis_dash;
    let custom_pat = custom_regex.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    if !any_filter && custom_pat.is_none() {
        return Err(AppError::new(CODE_ERROR, "[clean] 请选择至少一项筛选项或输入正则"));
    }

    // 编译自定义正则（若提供）
    let custom_re: Option<Regex> = if let Some(pat) = custom_pat.clone() {
        Some(Regex::new(&pat).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 正则无效: {e}")))?)
    } else {
        None
    };

    // 预编译各筛选项正则
    let re_half = if filters.half_punct {
        Some(Regex::new(r#"[,.!?;:()\[\]{}"'`]"#).unwrap())
    } else {
        None
    };
    // 特殊符号：不在白名单内的字符（白名单：中日韩+字母数字+常用中文标点+空白）
    let re_special = if filters.special_symbol {
        Some(Regex::new(r#"[^\u4e00-\u9fa5a-zA-Z0-9\s，。！？；：、（）【】《》“”‘’—…·]"#).unwrap())
    } else {
        None
    };
    let re_repeated = if filters.repeated_punct {
        Some(Regex::new(r#"([，。！？；：、]{2,}|[!?.,;:]{2,})"#).unwrap())
    } else {
        None
    };
    // 异常空白：行内连续2空格或 Tab（首尾已 trim，故不含首尾空格，但仍检测 \t 与 2空格）
    // 包含行内空格：例如 "a  b"（两个空格）即命中
    let re_whitespace = if filters.abnormal_whitespace {
        Some(Regex::new(r#" {2,}|\t|　"#).unwrap())
    } else {
        None
    };
    let re_ellipsis = if filters.ellipsis_dash {
        Some(Regex::new(r#"(\.\.\.|。。+|--|——)"#).unwrap())
    } else {
        None
    };

    let split_path = resolve_split_path(app, &id)?;
    if !split_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[clean] split.json 不存在，请先完成拆分"));
    }
    let data =
        fs::read_to_string(&split_path).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 读取 split.json 失败: {e}")))?;
    let split: SplitResult =
        serde_json::from_str(&data).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 解析 split.json 失败: {e}")))?;

    // 展开所有章节为 (vol, chap, contents)
    let mut chapters: Vec<(Option<i32>, i32, Vec<String>)> = Vec::new();
    if split.type_ == "pure_chapters" {
        if let Some(chs) = split.chapters {
            for ch in chs {
                chapters.push((None, ch.order, ch.contents));
            }
        }
    } else if let Some(vols) = split.volumes {
        for vol in vols {
            for ch in vol.chapters {
                chapters.push((Some(vol.order), ch.order, ch.contents));
            }
        }
    }

    let mut issues: Vec<CleanIssue> = Vec::new();
    let mut matched_set: HashSet<(Option<i32>, i32)> = HashSet::new();

    for (vol, chap_order, contents) in &chapters {
        for (idx, line) in contents.iter().enumerate() {
            // 半角标点
            if let Some(re) = &re_half {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "half_punct".to_string(),
                        message: "半角标点".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
            if let Some(re) = &re_special {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "special_symbol".to_string(),
                        message: "特殊符号".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
            if let Some(re) = &re_repeated {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "repeated_punct".to_string(),
                        message: "重复标点".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
            if let Some(re) = &re_whitespace {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "abnormal_whitespace".to_string(),
                        message: "异常空白".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
            if let Some(re) = &re_ellipsis {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "ellipsis_dash".to_string(),
                        message: "省略/破折号不规范".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
            if filters.unclosed_pairs && is_unclosed(line) {
                issues.push(CleanIssue {
                    volume_order: *vol,
                    chapter_order: *chap_order,
                    line_index: idx,
                    kind: "unclosed_pairs".to_string(),
                    message: "成对标点未闭合".to_string(),
                    context: line.clone(),
                    matched: None,
                });
                matched_set.insert((*vol, *chap_order));
            }
            if let Some(re) = &custom_re {
                if let Some(m) = re.find(line) {
                    issues.push(CleanIssue {
                        volume_order: *vol,
                        chapter_order: *chap_order,
                        line_index: idx,
                        kind: "custom_regex".to_string(),
                        message: "正则匹配".to_string(),
                        context: line.clone(),
                        matched: Some(m.as_str().to_string()),
                    });
                    matched_set.insert((*vol, *chap_order));
                }
            }
        }
    }

    let mut matched: Vec<CleanMatched> = matched_set
        .into_iter()
        .map(|(vol, chap)| CleanMatched {
            volume_order: vol,
            chapter_order: chap,
        })
        .collect();
    // 排序：先卷（None 最前）再章
    matched.sort_by(|a, b| {
        let av = a.volume_order.unwrap_or(0);
        let bv = b.volume_order.unwrap_or(0);
        if av != bv {
            return av.cmp(&bv);
        }
        a.chapter_order.cmp(&b.chapter_order)
    });
    // issues 按章→行排序
    issues.sort_by(|a, b| {
        let av = a.volume_order.unwrap_or(0);
        let bv = b.volume_order.unwrap_or(0);
        if av != bv {
            return av.cmp(&bv);
        }
        if a.chapter_order != b.chapter_order {
            return a.chapter_order.cmp(&b.chapter_order);
        }
        a.line_index.cmp(&b.line_index)
    });

    log::info!("[clean] filter {} -> matched {} issues {}", id, matched.len(), issues.len());

    Ok(CleanResult { matched, issues })
}

#[cfg(test)]
mod tests {
    use super::{CleanFilters, is_unclosed};

    #[test]
    fn test_unclosed() {
        assert!(is_unclosed("“你好"));
        assert!(!is_unclosed("“你好”"));
        assert!(is_unclosed("（你好"));
        assert!(!is_unclosed("（你好）"));
        assert!(is_unclosed(r#"He said "hello"#));
        assert!(!is_unclosed(r#"He said "hello""#));
        assert!(is_unclosed("‘test"));
    }

    #[test]
    fn test_filters_serde() {
        let json = r#"{"halfPunct":true,"specialSymbol":false,"unclosedPairs":true,"repeatedPunct":false,"abnormalWhitespace":true,"ellipsisDash":false}"#;
        let f: CleanFilters = serde_json::from_str(json).unwrap();
        assert!(f.half_punct);
        assert!(f.unclosed_pairs);
    }
}
