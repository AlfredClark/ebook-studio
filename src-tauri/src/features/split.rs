//! 拆分功能：将 content.txt 按 inspect 的分卷/分章识别逻辑拆为结构化 JSON
//!
//! 真相源：`APPDATA/Projects/<uuid>/sources/content.txt` → `APPDATA/Projects/<uuid>/split.json`
//! - 复用 inspect 的卷/章正则与中文数字解析，结构自动识别（volume_intro / volume_chapters / pure_chapters）
//! - `order` 为编号（优先解析结果，失败回退按出现顺序 1..n），`title` 为去编号纯标题（trim）
//! - 多行文本统一按行 `trim` 并过滤空行存为 `string[]`
//! - 写前所有字符串已 trim，落盘为 pretty JSON

#![allow(clippy::collapsible_if, clippy::unnecessary_sort_by)]

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::cores::response::{AppError, AppResult, CODE_ERROR};

const PROJECTS_DIR: &str = "Projects";
const SOURCES_DIR: &str = "sources";
const CONTENT_NAME: &str = "content.txt";
const SPLIT_NAME: &str = "split.json";

const DEFAULT_VOL_RE: &str = r"^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*卷\s*[:：]?\s*(.*)$";
const DEFAULT_CHAP_RE: &str = r"^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*章\s*(.*)$";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitChapter {
    pub order: i32,
    pub title: String,
    pub contents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitVolume {
    pub order: i32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro: Option<Vec<String>>,
    pub chapters: Vec<SplitChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitResult {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<SplitVolume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapters: Option<Vec<SplitChapter>>,
}

fn parse_chapter_number(s: &str) -> Option<i32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        return t.parse::<i32>().ok().filter(|&v| v > 0);
    }
    let norm = t.replace('兩', "两").replace('〇', "零");
    use chinese_number::{ChineseCountMethod, ChineseToNumber};
    let n: i32 = norm.to_number(ChineseCountMethod::TenThousand).ok()?;
    if n < 1 { None } else { Some(n) }
}

fn sanitize_identifier(identifier: &str) -> AppResult<String> {
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    uuid::Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[split] 非法 identifier"))?;
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[split] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn resolve_content_path(app: &AppHandle, identifier: &str) -> AppResult<PathBuf> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[split] resolve Projects dir 失败: {e}")))?;
    Ok(base.join(&uuid).join(SOURCES_DIR).join(CONTENT_NAME))
}

fn resolve_split_path(app: &AppHandle, identifier: &str) -> AppResult<PathBuf> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[split] resolve Projects dir 失败: {e}")))?;
    Ok(base.join(&uuid).join(SPLIT_NAME))
}

fn read_content_text(path: &Path) -> AppResult<String> {
    if !path.exists() {
        return Err(AppError::new(CODE_ERROR, "[split] content 文件不存在"));
    }
    let bytes = fs::read(path).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 读取 content 失败: {e}")))?;
    if bytes.len() > 100 * 1024 * 1024 {
        return Err(AppError::new(CODE_ERROR, "[split] 文件过大（>100MB）"));
    }
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        Ok(s)
    } else {
        let (cow, _, had_errors) = encoding_rs::GBK.decode(&bytes);
        if !had_errors {
            Ok(cow.into_owned())
        } else {
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct RawVolume {
    num_str: String,
    num: Option<i32>,
    title: String,
    line_no: u32,
}

#[allow(dead_code)]
#[derive(Clone)]
struct RawChapter {
    num_str: String,
    num: Option<i32>,
    title: String,
    line_no: u32,
    volume_idx: Option<usize>,
}

/// 读取已存在的 split.json，若不存在返回 None
pub(crate) fn get_split_content(app: &AppHandle, identifier: &str) -> AppResult<Option<SplitResult>> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[split] identifier 不能为空"));
    }
    let split_path = resolve_split_path(app, &id)?;
    if !split_path.exists() {
        return Ok(None);
    }
    let data =
        fs::read_to_string(&split_path).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 读取 split.json 失败: {e}")))?;
    let result: SplitResult =
        serde_json::from_str(&data).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 解析 split.json 失败: {e}")))?;
    Ok(Some(result))
}

/// 执行拆分：解析 content.txt → 生成 SplitResult → 写 split.json → 返回结果
pub(crate) fn split_content(app: &AppHandle, identifier: &str) -> AppResult<SplitResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[split] identifier 不能为空"));
    }
    let content_path = resolve_content_path(app, &id)?;
    let text = read_content_text(&content_path)?;
    let lines: Vec<(u32, String)> = text.lines().enumerate().map(|(i, l)| (i as u32 + 1, l.to_string())).collect();
    let total_lines = lines.len();

    let vol_re = Regex::new(DEFAULT_VOL_RE).expect("[split] 默认卷正则无效");
    let chap_re = Regex::new(DEFAULT_CHAP_RE).expect("[split] 默认章正则无效");
    let book_re = Regex::new(r"^\s*(书名|作者)\s*[:：]").unwrap();

    let mut raw_volumes: Vec<RawVolume> = Vec::new();
    let mut raw_chapters: Vec<RawChapter> = Vec::new();

    for (line_no, raw) in &lines {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if book_re.is_match(trimmed) {
            continue;
        }
        if let Some(caps) = vol_re.captures(trimmed) {
            let num_str = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let title = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let num = parse_chapter_number(&num_str);
            raw_volumes.push(RawVolume {
                num_str,
                num,
                title,
                line_no: *line_no,
            });
            continue;
        }
        if let Some(caps) = chap_re.captures(trimmed) {
            let num_str = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let title = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let num = parse_chapter_number(&num_str);
            raw_chapters.push(RawChapter {
                num_str,
                num,
                title,
                line_no: *line_no,
                volume_idx: None,
            });
            continue;
        }
    }

    if raw_chapters.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[split] 未识别到任何章节，无法拆分"));
    }

    raw_volumes.sort_by_key(|v| v.line_no);
    raw_chapters.sort_by_key(|c| c.line_no);
    for chap in &mut raw_chapters {
        let mut best: Option<usize> = None;
        for (idx, vol) in raw_volumes.iter().enumerate() {
            if vol.line_no < chap.line_no {
                best = Some(idx);
            } else {
                break;
            }
        }
        chap.volume_idx = best;
    }

    // 卷简介判定：卷后至下一卷/章间非空正文段即为 intro（用于结构自动识别）
    let vol_lines: HashSet<u32> = raw_volumes.iter().map(|v| v.line_no).collect();
    let chap_lines: HashSet<u32> = raw_chapters.iter().map(|c| c.line_no).collect();
    let mut intro_count = 0usize;
    // 同时收集每卷 intro 行（供后续落盘）
    let mut volume_intros: Vec<Vec<String>> = vec![Vec::new(); raw_volumes.len()];
    for idx in 0..raw_volumes.len() {
        let vol_line = raw_volumes[idx].line_no;
        let mut candidate: Option<u32> = None;
        for c in &raw_chapters {
            if c.line_no > vol_line && (candidate.is_none() || c.line_no < candidate.unwrap()) {
                candidate = Some(c.line_no);
            }
        }
        for v in raw_volumes.iter().skip(idx + 1) {
            if v.line_no > vol_line {
                if candidate.is_none() || v.line_no < candidate.unwrap() {
                    candidate = Some(v.line_no);
                }
                break;
            }
        }
        let start = vol_line + 1;
        let end = candidate.unwrap_or((total_lines as u32) + 1);
        let mut intro_lines: Vec<String> = Vec::new();
        for (ln, raw) in &lines {
            if *ln >= start && *ln < end {
                let t = raw.trim();
                if t.is_empty() {
                    continue;
                }
                if vol_lines.contains(ln) || chap_lines.contains(ln) || book_re.is_match(t) {
                    continue;
                }
                intro_lines.push(t.to_string());
            }
        }
        if !intro_lines.is_empty() {
            intro_count += 1;
        }
        volume_intros[idx] = intro_lines;
    }

    let vol_count = raw_volumes.len();
    let chap_count = raw_chapters.len();

    // 自动识别结构，与 inspect.rs 保持一致
    let auto_detected = if chap_count == 0 {
        "pure_chapters"
    } else if vol_count > 0 && intro_count > 0 {
        "volume_intro"
    } else if vol_count > 0 {
        "volume_chapters"
    } else {
        "pure_chapters"
    };

    // 生成 SplitResult
    let result = if auto_detected == "pure_chapters" {
        // 纯章节：平铺
        let mut chapters: Vec<SplitChapter> = Vec::new();
        // 按行号排序的 chapters 已保证顺序
        for (idx, c) in raw_chapters.iter().enumerate() {
            let order = c.num.unwrap_or((idx as i32) + 1);
            // 查找该章的内容边界：下一章或下一卷（纯章节场景下卷为空，故仅下一章）
            let next_boundary = {
                let mut cand: Option<u32> = None;
                for nxt in &raw_chapters {
                    if nxt.line_no > c.line_no && (cand.is_none() || nxt.line_no < cand.unwrap()) {
                        cand = Some(nxt.line_no);
                    }
                }
                for v in &raw_volumes {
                    if v.line_no > c.line_no && (cand.is_none() || v.line_no < cand.unwrap()) {
                        cand = Some(v.line_no);
                    }
                }
                cand
            };
            let start = c.line_no + 1;
            let end = next_boundary.unwrap_or((total_lines as u32) + 1);
            let mut contents: Vec<String> = Vec::new();
            for (ln, raw) in &lines {
                if *ln >= start && *ln < end {
                    let t = raw.trim();
                    if t.is_empty() {
                        continue;
                    }
                    if vol_lines.contains(ln) || chap_lines.contains(ln) || book_re.is_match(t) {
                        continue;
                    }
                    contents.push(t.to_string());
                }
            }
            // title 已 trim
            chapters.push(SplitChapter {
                order,
                title: c.title.trim().to_string(),
                contents,
            });
        }
        SplitResult {
            type_: "pure_chapters".to_string(),
            volumes: None,
            chapters: Some(chapters),
        }
    } else {
        // 分卷 + 章节（含或不含简介）
        let is_volume_intro = auto_detected == "volume_intro";
        let mut volumes: Vec<SplitVolume> = Vec::new();
        // 处理卷外章节（volume_idx==None）→ 归入首卷头部，若无卷则已在 pure 分支
        let outside_chapters: Vec<RawChapter> = raw_chapters.iter().filter(|c| c.volume_idx.is_none()).cloned().collect();
        for (vi, vol) in raw_volumes.iter().enumerate() {
            let order = vol.num.unwrap_or((vi as i32) + 1);
            let intro = if is_volume_intro {
                let iv = volume_intros[vi].clone();
                if iv.is_empty() { None } else { Some(iv) }
            } else {
                None
            };
            // 收集属于该卷的章节
            let mut vol_chaps_raw: Vec<RawChapter> =
                raw_chapters.iter().filter(|c| c.volume_idx == Some(vi)).cloned().collect();
            vol_chaps_raw.sort_by_key(|c| c.line_no);
            // 若为首卷且存在卷外章节，插入到头部
            if vi == 0 && !outside_chapters.is_empty() {
                let mut merged = outside_chapters.clone();
                merged.extend(vol_chaps_raw);
                merged.sort_by_key(|c| c.line_no);
                vol_chaps_raw = merged;
            }
            let mut chapters: Vec<SplitChapter> = Vec::new();
            for (ci, c) in vol_chaps_raw.iter().enumerate() {
                let chap_order = c.num.unwrap_or((ci as i32) + 1);
                let next_boundary = {
                    let mut cand: Option<u32> = None;
                    for nxt in &raw_chapters {
                        if nxt.line_no > c.line_no && (cand.is_none() || nxt.line_no < cand.unwrap()) {
                            cand = Some(nxt.line_no);
                        }
                    }
                    for v in &raw_volumes {
                        if v.line_no > c.line_no && (cand.is_none() || v.line_no < cand.unwrap()) {
                            cand = Some(v.line_no);
                        }
                    }
                    cand
                };
                let start = c.line_no + 1;
                let end = next_boundary.unwrap_or((total_lines as u32) + 1);
                let mut contents: Vec<String> = Vec::new();
                for (ln, raw) in &lines {
                    if *ln >= start && *ln < end {
                        let t = raw.trim();
                        if t.is_empty() {
                            continue;
                        }
                        if vol_lines.contains(ln) || chap_lines.contains(ln) || book_re.is_match(t) {
                            continue;
                        }
                        contents.push(t.to_string());
                    }
                }
                chapters.push(SplitChapter {
                    order: chap_order,
                    title: c.title.trim().to_string(),
                    contents,
                });
            }
            volumes.push(SplitVolume {
                order,
                title: vol.title.trim().to_string(),
                intro,
                chapters,
            });
        }
        SplitResult {
            type_: auto_detected.to_string(),
            volumes: Some(volumes),
            chapters: None,
        }
    };

    // 落盘
    let split_path = resolve_split_path(app, &id)?;
    let json =
        serde_json::to_string_pretty(&result).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 序列化失败: {e}")))?;
    fs::write(&split_path, json).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 写入 split.json 失败: {e}")))?;

    // 更新 metadata modified（与 inspect reorder 同步）
    {
        let uuid = sanitize_identifier(&id)?;
        let base = app
            .path()
            .resolve(PROJECTS_DIR, BaseDirectory::AppData)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[split] resolve dir 失败: {e}")))?;
        let meta_path = base.join(&uuid).join("metadata.json");
        if meta_path.exists() {
            if let Ok(data) = fs::read_to_string(&meta_path) {
                if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&data) {
                    let now = Utc::now();
                    let now_rfc = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                    let now_ms = now.timestamp_millis();
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("modified".to_string(), serde_json::Value::String(now_rfc));
                        obj.insert(
                            "modifiedMs".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(now_ms)),
                        );
                        obj.insert(
                            "modified_ms".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(now_ms)),
                        );
                    }
                    if let Ok(s) = serde_json::to_string_pretty(&v) {
                        let _ = fs::write(&meta_path, s);
                    }
                }
            }
        }
    }

    log::info!("[split] split {} -> {}", id, auto_detected);
    Ok(result)
}

/// 保存章节内容（落盘）：根据 volumeOrder/chapterOrder 定位章节并覆写 contents
pub(crate) fn save_split_chapter(
    app: &AppHandle,
    identifier: &str,
    volume_order: Option<i32>,
    chapter_order: i32,
    contents: Vec<String>,
) -> AppResult<SplitResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[split] identifier 不能为空"));
    }
    // 规范化 contents：trim + 过滤空行（与拆分一致）
    let normalized: Vec<String> = contents
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let split_path = resolve_split_path(app, &id)?;
    if !split_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[split] split.json 不存在，请先拆分"));
    }
    let data =
        fs::read_to_string(&split_path).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 读取 split.json 失败: {e}")))?;
    let mut result: SplitResult =
        serde_json::from_str(&data).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 解析 split.json 失败: {e}")))?;

    // 定位并更新
    let mut found = false;
    if result.type_ == "pure_chapters" {
        if let Some(chaps) = result.chapters.as_mut() {
            for ch in chaps.iter_mut() {
                if ch.order == chapter_order {
                    ch.contents = normalized.clone();
                    found = true;
                    break;
                }
            }
        }
    } else {
        // volume_*：需 volumeOrder
        let vol_order = volume_order.ok_or_else(|| AppError::new(CODE_ERROR, "[split] 分卷结构下需提供 volumeOrder"))?;
        if let Some(vols) = result.volumes.as_mut() {
            for vol in vols.iter_mut() {
                if vol.order == vol_order {
                    for ch in vol.chapters.iter_mut() {
                        if ch.order == chapter_order {
                            ch.contents = normalized.clone();
                            found = true;
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }

    if !found {
        return Err(AppError::new(CODE_ERROR, "[split] 未找到对应章节"));
    }

    let json =
        serde_json::to_string_pretty(&result).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 序列化失败: {e}")))?;
    fs::write(&split_path, json).map_err(|e| AppError::new(CODE_ERROR, format!("[split] 写入 split.json 失败: {e}")))?;

    // 同步更新 metadata modified
    {
        let uuid = sanitize_identifier(&id)?;
        let base = app
            .path()
            .resolve(PROJECTS_DIR, BaseDirectory::AppData)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[split] resolve dir 失败: {e}")))?;
        let meta_path = base.join(&uuid).join("metadata.json");
        if meta_path.exists() {
            if let Ok(data) = fs::read_to_string(&meta_path) {
                if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&data) {
                    let now = Utc::now();
                    let now_rfc = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                    let now_ms = now.timestamp_millis();
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("modified".to_string(), serde_json::Value::String(now_rfc));
                        obj.insert(
                            "modifiedMs".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(now_ms)),
                        );
                        obj.insert(
                            "modified_ms".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(now_ms)),
                        );
                    }
                    if let Ok(s) = serde_json::to_string_pretty(&v) {
                        let _ = fs::write(&meta_path, s);
                    }
                }
            }
        }
    }

    log::info!(
        "[split] save chapter {}-{} for {}",
        volume_order.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()),
        chapter_order,
        id
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::parse_chapter_number;

    #[test]
    fn test_cn_number() {
        assert_eq!(parse_chapter_number("一"), Some(1));
        assert_eq!(parse_chapter_number("十"), Some(10));
        assert_eq!(parse_chapter_number("十二"), Some(12));
        assert_eq!(parse_chapter_number("二十"), Some(20));
        assert_eq!(parse_chapter_number("100"), Some(100));
        assert_eq!(parse_chapter_number("两千"), Some(2000));
    }

    #[test]
    fn test_split_result_serde() {
        let json = r#"{"type":"volume_intro","volumes":[{"order":1,"title":"卷标题一","intro":["简介一行"],"chapters":[{"order":1,"title":"风起天南","contents":["正文一行"]}]}]}"#;
        let v: super::SplitResult = serde_json::from_str(json).unwrap();
        assert_eq!(v.type_, "volume_intro");
        assert!(v.volumes.is_some());
        let out = serde_json::to_string(&v).unwrap();
        assert!(out.contains("volume_intro"));
    }

    #[test]
    fn test_pure_serde() {
        let json = r#"{"type":"pure_chapters","chapters":[{"order":1,"title":"章一","contents":["a","b"]}]}"#;
        let v: super::SplitResult = serde_json::from_str(json).unwrap();
        assert_eq!(v.type_, "pure_chapters");
        assert!(v.chapters.is_some());
        assert!(v.volumes.is_none());
    }
}
