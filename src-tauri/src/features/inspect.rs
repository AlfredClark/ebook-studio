//! 检查功能：content.txt 分卷分章结构校验
//!
//! 支持三种结构：
//!
//! - `volume_chapters`：分卷+章节（无简介）如 第一卷/第一章...
//! - `pure_chapters`：纯章节
//! - `volume_intro`：分卷简介（卷后紧跟简介段）
//!
//! 章节/卷编号支持阿拉伯与中文数字，自动识别或强制指定

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StructureType {
    #[serde(rename = "volume_chapters")]
    VolumeChapters,
    #[serde(rename = "pure_chapters")]
    PureChapters,
    #[serde(rename = "volume_intro")]
    VolumeIntro,
    #[serde(rename = "auto")]
    Auto,
}

impl StructureType {
    fn as_str(&self) -> &'static str {
        match self {
            StructureType::VolumeChapters => "volume_chapters",
            StructureType::PureChapters => "pure_chapters",
            StructureType::VolumeIntro => "volume_intro",
            StructureType::Auto => "auto",
        }
    }
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "volume_chapters" => Some(StructureType::VolumeChapters),
            "pure_chapters" => Some(StructureType::PureChapters),
            "volume_intro" => Some(StructureType::VolumeIntro),
            "auto" => Some(StructureType::Auto),
            _ => None,
        }
    }
}

const DEFAULT_VOL_RE: &str = r"^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*卷\s*[:：]?\s*(.*)$";
const DEFAULT_CHAP_RE: &str = r"^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*章\s*(.*)$";

fn compile_regex(custom: Option<String>, default_pat: &str) -> AppResult<Regex> {
    if let Some(pat) = custom.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        Regex::new(&pat).map_err(|e| AppError::new(CODE_ERROR, format!("[inspect] 正则无效: {e}")))
    } else {
        // 默认必合法
        Ok(Regex::new(default_pat).expect("[inspect] 默认正则无效"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectRequest {
    pub identifier: String,
    #[serde(default)]
    pub structure: Option<String>, // auto | volume_chapters | pure_chapters | volume_intro
    #[serde(default)]
    pub volume_regex: Option<String>,
    #[serde(default)]
    pub chapter_regex: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
    pub index: usize,
    pub num: Option<i32>,
    pub raw_num: Option<String>,
    pub title: String,
    pub line_no: u32,
    pub has_intro: bool,
    pub intro: Option<String>,
    pub chapter_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterInfo {
    pub index: usize,
    pub num: Option<i32>,
    pub raw_num: String,
    pub title: String,
    pub line_no: u32,
    pub volume_index: Option<usize>,
    pub volume_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectIssue {
    pub line_no: Option<u32>,
    pub kind: String, // missing | duplicate | out_of_order | invalid_number | empty_volume | no_chapters
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectStats {
    pub total_lines: usize,
    pub total_volumes: usize,
    pub total_chapters: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectResult {
    pub detected_structure: String,
    pub requested_structure: String,
    pub volumes: Vec<VolumeInfo>,
    pub chapters: Vec<ChapterInfo>,
    pub issues: Vec<InspectIssue>,
    pub stats: InspectStats,
    pub abs_path: Option<String>,
}

// ---------------- 中文数字转 int（委托 chinese-number） ----------------
fn parse_chapter_number(s: &str) -> Option<i32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        return t.parse::<i32>().ok().filter(|&v| v > 0);
    }
    // 预处理：crate 已支持 两/〇，但保留幂等归一
    let norm = t.replace('兩', "两").replace('〇', "零");
    use chinese_number::{ChineseCountMethod, ChineseToNumber};
    let n: i32 = norm.to_number(ChineseCountMethod::TenThousand).ok()?;
    if n < 1 { None } else { Some(n) }
}

fn format_number(n: i32, raw: &str) -> String {
    // 保留原风格：全阿拉伯则用阿拉伯，否则用中文
    if raw.chars().all(|c| c.is_ascii_digit()) {
        n.to_string()
    } else {
        use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
        match n.to_chinese(ChineseVariant::Simple, ChineseCase::Lower, ChineseCountMethod::TenThousand) {
            Ok(s) => s,
            Err(_) => n.to_string(),
        }
    }
}

fn sanitize_identifier(identifier: &str) -> AppResult<String> {
    // 复用 projects 的 sanitize 逻辑
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    uuid::Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[inspect] 非法 identifier"))?;
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[inspect] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn resolve_content_path(app: &AppHandle, identifier: &str) -> AppResult<std::path::PathBuf> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[inspect] resolve Projects dir 失败: {e}")))?;
    let path = base.join(&uuid).join(SOURCES_DIR).join(CONTENT_NAME);
    Ok(path)
}

fn read_content_text(path: &Path) -> AppResult<String> {
    if !path.exists() {
        return Err(AppError::new(CODE_ERROR, "[inspect] content 文件不存在"));
    }
    let bytes = fs::read(path).map_err(|e| AppError::new(CODE_ERROR, format!("[inspect] 读取 content 失败: {e}")))?;
    if bytes.len() > 100 * 1024 * 1024 {
        return Err(AppError::new(CODE_ERROR, "[inspect] 文件过大（>100MB）"));
    }
    // 优先 UTF-8，失败用 GBK/GB18030 尝试
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        Ok(s)
    } else {
        // 尝试 GBK
        let (cow, _, had_errors) = encoding_rs::GBK.decode(&bytes);
        if !had_errors {
            Ok(cow.into_owned())
        } else {
            // 最后 lossy
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}

#[derive(Clone)]
struct RawVolume {
    num_str: String,
    num: Option<i32>,
    title: String,
    line_no: u32,
    intro: Option<String>,
    has_intro: bool,
}

#[derive(Clone)]
struct RawChapter {
    num_str: String,
    num: Option<i32>,
    title: String,
    line_no: u32,
    volume_idx: Option<usize>,
}

pub(crate) fn inspect_content(app: &AppHandle, req: InspectRequest) -> AppResult<InspectResult> {
    let identifier = req.identifier.trim().to_string();
    if identifier.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[inspect] identifier 不能为空"));
    }
    let requested = req
        .structure
        .as_deref()
        .map(|s| StructureType::from_str(s).unwrap_or(StructureType::Auto))
        .unwrap_or(StructureType::Auto);

    let content_path = resolve_content_path(app, &identifier)?;
    let abs_path = content_path.to_string_lossy().to_string();
    let text = read_content_text(&content_path)?;
    let lines: Vec<(u32, String)> = text.lines().enumerate().map(|(i, l)| (i as u32 + 1, l.to_string())).collect();
    let total_lines = lines.len();

    // 正则：卷 / 章（支持自定义，空则回退默认）
    let vol_re = compile_regex(req.volume_regex, DEFAULT_VOL_RE)?;
    let chap_re = compile_regex(req.chapter_regex, DEFAULT_CHAP_RE)?;
    // 书名/作者等头部可忽略
    let book_re = Regex::new(r"^\s*(书名|作者)\s*[:：]").unwrap();

    let mut raw_volumes: Vec<RawVolume> = Vec::new();
    let mut raw_chapters: Vec<RawChapter> = Vec::new();

    // 第一遍：提取卷章 token
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
                intro: None,
                has_intro: false,
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
        // 非卷非章：视为正文，暂不处理
    }

    // 为章节关联卷索引（按行号最近的前一卷）
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

    // 卷简介判定：卷后到下一卷/章之间的非空正文段
    // 构建 line_no -> is_vol/is_chap 快速查找
    let vol_lines: HashSet<u32> = raw_volumes.iter().map(|v| v.line_no).collect();
    let chap_lines: HashSet<u32> = raw_chapters.iter().map(|c| c.line_no).collect();
    for idx in 0..raw_volumes.len() {
        let vol_line = raw_volumes[idx].line_no;
        let next_boundary = {
            // 取卷后第一个章或下一卷的最小行号
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
            candidate
        };
        let start = vol_line + 1;
        let end = next_boundary.unwrap_or((total_lines as u32) + 1);
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
            raw_volumes[idx].has_intro = true;
            raw_volumes[idx].intro = Some(intro_lines.join("\n"));
        }
    }

    let vol_count = raw_volumes.len();
    let chap_count = raw_chapters.len();
    let intro_count = raw_volumes.iter().filter(|v| v.has_intro).count();

    let auto_detected = if chap_count == 0 {
        // 无章节，仍按纯章节报告，后续发 no_chapters
        StructureType::PureChapters
    } else if vol_count > 0 && intro_count > 0 {
        StructureType::VolumeIntro
    } else if vol_count > 0 {
        StructureType::VolumeChapters
    } else {
        StructureType::PureChapters
    };

    let effective = if requested == StructureType::Auto {
        auto_detected.clone()
    } else {
        requested.clone()
    };

    // 构建输出结构
    let mut volumes_out: Vec<VolumeInfo> = Vec::new();
    for (i, v) in raw_volumes.iter().enumerate() {
        let chap_cnt = raw_chapters.iter().filter(|c| c.volume_idx == Some(i)).count();
        volumes_out.push(VolumeInfo {
            index: i,
            num: v.num,
            raw_num: Some(v.num_str.clone()),
            title: v.title.clone(),
            line_no: v.line_no,
            has_intro: v.has_intro,
            intro: v.intro.clone(),
            chapter_count: chap_cnt,
        });
    }
    let mut chapters_out: Vec<ChapterInfo> = Vec::new();
    for (i, c) in raw_chapters.iter().enumerate() {
        let vol_title = c.volume_idx.and_then(|idx| raw_volumes.get(idx).map(|v| v.title.clone()));
        chapters_out.push(ChapterInfo {
            index: i,
            num: c.num,
            raw_num: c.num_str.clone(),
            title: c.title.clone(),
            line_no: c.line_no,
            volume_index: c.volume_idx,
            volume_title: vol_title,
        });
    }

    let mut issues: Vec<InspectIssue> = Vec::new();

    if chap_count == 0 {
        issues.push(InspectIssue {
            line_no: None,
            kind: "no_chapters".to_string(),
            message: "未识别到任何章节".to_string(),
            expected: None,
            actual: None,
            context: None,
        });
    }

    // 卷编号校验
    {
        let mut expected: i32 = 1;
        let mut seen: HashSet<i32> = HashSet::new();
        let mut prev: Option<i32> = None;
        for v in &raw_volumes {
            if let Some(n) = v.num {
                if seen.contains(&n) {
                    issues.push(InspectIssue {
                        line_no: Some(v.line_no),
                        kind: "duplicate".to_string(),
                        message: format!("卷编号重复：第{}卷（行 {}）", v.num_str, v.line_no),
                        expected: None,
                        actual: Some(v.num_str.clone()),
                        context: Some(v.title.clone()),
                    });
                } else if let Some(p) = prev {
                    if n < p {
                        issues.push(InspectIssue {
                            line_no: Some(v.line_no),
                            kind: "out_of_order".to_string(),
                            message: format!("卷编号乱序：第{}卷 出现在第{}卷之后（行 {}）", v.num_str, p, v.line_no),
                            expected: Some(format!(">{}", p)),
                            actual: Some(v.num_str.clone()),
                            context: Some(v.title.clone()),
                        });
                    } else if n > expected {
                        // 缺失 expected..n-1
                        for miss in expected..n {
                            issues.push(InspectIssue {
                                line_no: Some(v.line_no),
                                kind: "missing".to_string(),
                                message: format!("卷缺失：第{}卷", miss),
                                expected: Some(miss.to_string()),
                                actual: Some(v.num_str.clone()),
                                context: None,
                            });
                        }
                    }
                } else {
                    // 首卷
                    if n != 1 {
                        for miss in 1..n {
                            issues.push(InspectIssue {
                                line_no: Some(v.line_no),
                                kind: "missing".to_string(),
                                message: format!("卷缺失：第{}卷", miss),
                                expected: Some(miss.to_string()),
                                actual: Some(v.num_str.clone()),
                                context: None,
                            });
                        }
                    }
                }
                seen.insert(n);
                prev = Some(n);
                if n >= expected {
                    expected = n + 1;
                }
            } else {
                issues.push(InspectIssue {
                    line_no: Some(v.line_no),
                    kind: "invalid_number".to_string(),
                    message: format!("卷编号无法解析：{}（行 {}）", v.num_str, v.line_no),
                    expected: None,
                    actual: Some(v.num_str.clone()),
                    context: Some(v.title.clone()),
                });
            }
        }
    }

    // 章节编号含无效数字先报
    for c in &raw_chapters {
        if c.num.is_none() {
            issues.push(InspectIssue {
                line_no: Some(c.line_no),
                kind: "invalid_number".to_string(),
                message: format!("章节编号无法解析：第{}章（行 {}）", c.num_str, c.line_no),
                expected: None,
                actual: Some(c.num_str.clone()),
                context: Some(c.title.clone()),
            });
        }
    }

    // 空卷
    for (i, v) in raw_volumes.iter().enumerate() {
        let cnt = raw_chapters.iter().filter(|c| c.volume_idx == Some(i)).count();
        if cnt == 0 {
            issues.push(InspectIssue {
                line_no: Some(v.line_no),
                kind: "empty_volume".to_string(),
                message: format!("空卷：第{}卷 {}（行 {}）下无任何章节", v.num_str, v.title, v.line_no),
                expected: None,
                actual: None,
                context: Some(v.title.clone()),
            });
        }
    }

    // 根据结构进行章节连续性校验
    match effective {
        StructureType::PureChapters => {
            // 全局连续
            let mut expected: i32 = 1;
            let mut seen: HashSet<i32> = HashSet::new();
            let mut prev: Option<i32> = None;
            for c in &raw_chapters {
                if let Some(n) = c.num {
                    if seen.contains(&n) {
                        issues.push(InspectIssue {
                            line_no: Some(c.line_no),
                            kind: "duplicate".to_string(),
                            message: format!("章节编号重复：第{}章 {}（行 {}）", c.num_str, c.title, c.line_no),
                            expected: None,
                            actual: Some(c.num_str.clone()),
                            context: Some(c.title.clone()),
                        });
                        continue;
                    }
                    if let Some(p) = prev {
                        if n < p {
                            issues.push(InspectIssue {
                                line_no: Some(c.line_no),
                                kind: "out_of_order".to_string(),
                                message: format!("章节乱序：第{}章 出现在第{}章之后（行 {}）", c.num_str, p, c.line_no),
                                expected: Some(format!(">{}", p)),
                                actual: Some(c.num_str.clone()),
                                context: Some(c.title.clone()),
                            });
                        } else if n > expected {
                            for miss in expected..n {
                                issues.push(InspectIssue {
                                    line_no: Some(c.line_no),
                                    kind: "missing".to_string(),
                                    message: format!("章节缺失：第{}章", miss),
                                    expected: Some(miss.to_string()),
                                    actual: Some(c.num_str.clone()),
                                    context: None,
                                });
                            }
                        }
                    } else if n != 1 {
                        for miss in 1..n {
                            issues.push(InspectIssue {
                                line_no: Some(c.line_no),
                                kind: "missing".to_string(),
                                message: format!("章节缺失：第{}章", miss),
                                expected: Some(miss.to_string()),
                                actual: Some(c.num_str.clone()),
                                context: None,
                            });
                        }
                    }
                    seen.insert(n);
                    prev = Some(n);
                    if n >= expected {
                        expected = n + 1;
                    }
                }
            }
        }
        StructureType::VolumeChapters | StructureType::VolumeIntro => {
            // 判断重置 vs 连续：第二卷首章是否为1
            let mut first_nums: Vec<Option<i32>> = Vec::new();
            for (i, _) in raw_volumes.iter().enumerate() {
                let first = raw_chapters.iter().find(|c| c.volume_idx == Some(i));
                if let Some(c) = first {
                    first_nums.push(c.num);
                } else {
                    first_nums.push(None);
                }
            }
            let is_reset = if raw_volumes.len() <= 1 {
                true
            } else {
                // 若所有有章节的卷首章均为1 => 重置，否则连续
                let mut all_one = true;
                let mut has_chapter_vols = 0;
                for v in first_nums.iter().flatten() {
                    has_chapter_vols += 1;
                    if *v != 1 {
                        all_one = false;
                    }
                }
                if has_chapter_vols == 0 { true } else { all_one }
            };
            if is_reset {
                // 按卷独立校验 1..k
                for (vi, _) in raw_volumes.iter().enumerate() {
                    let mut chaps: Vec<&RawChapter> = raw_chapters.iter().filter(|c| c.volume_idx == Some(vi)).collect();
                    chaps.sort_by_key(|c| c.line_no);
                    let mut expected: i32 = 1;
                    let mut seen: HashSet<i32> = HashSet::new();
                    let mut prev: Option<i32> = None;
                    for c in chaps {
                        if let Some(n) = c.num {
                            if seen.contains(&n) {
                                issues.push(InspectIssue {
                                    line_no: Some(c.line_no),
                                    kind: "duplicate".to_string(),
                                    message: format!(
                                        "章节编号重复（卷内）：第{}章 {}（行 {}，卷 {}）",
                                        c.num_str,
                                        c.title,
                                        c.line_no,
                                        vi + 1
                                    ),
                                    expected: None,
                                    actual: Some(c.num_str.clone()),
                                    context: Some(c.title.clone()),
                                });
                                continue;
                            }
                            if let Some(p) = prev {
                                if n < p {
                                    issues.push(InspectIssue {
                                        line_no: Some(c.line_no),
                                        kind: "out_of_order".to_string(),
                                        message: format!(
                                            "章节乱序（卷内）：第{}章 出现在第{}章之后（行 {}）",
                                            c.num_str, p, c.line_no
                                        ),
                                        expected: Some(format!(">{}", p)),
                                        actual: Some(c.num_str.clone()),
                                        context: Some(c.title.clone()),
                                    });
                                } else if n > expected {
                                    for miss in expected..n {
                                        issues.push(InspectIssue {
                                            line_no: Some(c.line_no),
                                            kind: "missing".to_string(),
                                            message: format!("章节缺失（卷内卷{}）：第{}章", vi + 1, miss),
                                            expected: Some(miss.to_string()),
                                            actual: Some(c.num_str.clone()),
                                            context: None,
                                        });
                                    }
                                }
                            } else if n != 1 {
                                for miss in 1..n {
                                    issues.push(InspectIssue {
                                        line_no: Some(c.line_no),
                                        kind: "missing".to_string(),
                                        message: format!("章节缺失（卷内卷{}）：第{}章", vi + 1, miss),
                                        expected: Some(miss.to_string()),
                                        actual: Some(c.num_str.clone()),
                                        context: None,
                                    });
                                }
                            }
                            seen.insert(n);
                            prev = Some(n);
                            if n >= expected {
                                expected = n + 1;
                            }
                        }
                    }
                }
                // 额外：卷间重复在重置模式下不算全局重复，已在卷内处理；但仍需检测卷间标题重复？不需要
            } else {
                // 连续模式：全局校验
                let mut expected: i32 = 1;
                let mut seen: HashSet<i32> = HashSet::new();
                let mut prev: Option<i32> = None;
                for c in &raw_chapters {
                    if let Some(n) = c.num {
                        if seen.contains(&n) {
                            issues.push(InspectIssue {
                                line_no: Some(c.line_no),
                                kind: "duplicate".to_string(),
                                message: format!("章节编号重复：第{}章 {}（行 {}）", c.num_str, c.title, c.line_no),
                                expected: None,
                                actual: Some(c.num_str.clone()),
                                context: Some(c.title.clone()),
                            });
                            continue;
                        }
                        if let Some(p) = prev {
                            if n < p {
                                issues.push(InspectIssue {
                                    line_no: Some(c.line_no),
                                    kind: "out_of_order".to_string(),
                                    message: format!("章节乱序：第{}章 出现在第{}章之后（行 {}）", c.num_str, p, c.line_no),
                                    expected: Some(format!(">{}", p)),
                                    actual: Some(c.num_str.clone()),
                                    context: Some(c.title.clone()),
                                });
                            } else if n > expected {
                                for miss in expected..n {
                                    issues.push(InspectIssue {
                                        line_no: Some(c.line_no),
                                        kind: "missing".to_string(),
                                        message: format!("章节缺失：第{}章", miss),
                                        expected: Some(miss.to_string()),
                                        actual: Some(c.num_str.clone()),
                                        context: None,
                                    });
                                }
                            }
                        } else if n != 1 {
                            for miss in 1..n {
                                issues.push(InspectIssue {
                                    line_no: Some(c.line_no),
                                    kind: "missing".to_string(),
                                    message: format!("章节缺失：第{}章", miss),
                                    expected: Some(miss.to_string()),
                                    actual: Some(c.num_str.clone()),
                                    context: None,
                                });
                            }
                        }
                        seen.insert(n);
                        prev = Some(n);
                        if n >= expected {
                            expected = n + 1;
                        }
                    }
                }
            }
            // 纯卷间章节缺失已在对应模式处理；额外检查：章节在卷外（pure结构强制卷内，但 volume 结构允许卷前章节？）
            // 若结构为 volume 且存在无卷章节，报 issue
            if effective != StructureType::PureChapters {
                for c in &raw_chapters {
                    if c.volume_idx.is_none() {
                        issues.push(InspectIssue {
                            line_no: Some(c.line_no),
                            kind: "out_of_order".to_string(),
                            message: format!("章节位于卷外：第{}章 {}（行 {}）", c.num_str, c.title, c.line_no),
                            expected: None,
                            actual: Some(c.num_str.clone()),
                            context: Some(c.title.clone()),
                        });
                    }
                }
            }
        }
        StructureType::Auto => unreachable!(),
    }

    let result = InspectResult {
        detected_structure: auto_detected.as_str().to_string(),
        requested_structure: if requested == StructureType::Auto {
            auto_detected.as_str().to_string()
        } else {
            requested.as_str().to_string()
        },
        volumes: volumes_out,
        chapters: chapters_out,
        issues,
        stats: InspectStats {
            total_lines,
            total_volumes: vol_count,
            total_chapters: chap_count,
        },
        abs_path: Some(abs_path),
    };
    Ok(result)
}

/// 重整章节/卷编号：保留标题，仅重写编号为连续递增（保留原阿拉伯/中文风格）
pub(crate) fn reorder_chapters(app: &AppHandle, req: InspectRequest) -> AppResult<InspectResult> {
    let identifier = req.identifier.trim().to_string();
    if identifier.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[inspect] identifier 不能为空"));
    }
    // 复用与 inspect 相同的解析，得到 raw 结构与重整映射
    let content_path = resolve_content_path(app, &identifier)?;
    let text = read_content_text(&content_path)?;
    let lines: Vec<(u32, String)> = text.lines().enumerate().map(|(i, l)| (i as u32 + 1, l.to_string())).collect();
    let vol_re = compile_regex(req.volume_regex.clone(), DEFAULT_VOL_RE)?;
    let chap_re = compile_regex(req.chapter_regex.clone(), DEFAULT_CHAP_RE)?;
    let book_re = Regex::new(r"^\s*(书名|作者)\s*[:：]").unwrap();

    let mut raw_volumes: Vec<RawVolume> = Vec::new();
    let mut raw_chapters: Vec<RawChapter> = Vec::new();
    for (line_no, raw) in &lines {
        let trimmed = raw.trim();
        if trimmed.is_empty() || book_re.is_match(trimmed) {
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
                intro: None,
                has_intro: false,
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
        }
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
    // 判定有效结构与重置/连续（复用 inspect 逻辑）
    let vol_count = raw_volumes.len();
    let chap_count = raw_chapters.len();
    if chap_count == 0 {
        return Err(AppError::new(CODE_ERROR, "[inspect] 无章节可重整"));
    }
    // intro 判定仅用于 auto 结构选择，但重整时仍需按实际 effective
    let vol_lines: HashSet<u32> = raw_volumes.iter().map(|v| v.line_no).collect();
    let chap_lines: HashSet<u32> = raw_chapters.iter().map(|c| c.line_no).collect();
    let mut intro_count = 0;
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
        let total_lines = lines.len() as u32;
        let start = vol_line + 1;
        let end = candidate.unwrap_or(total_lines + 1);
        let mut has_intro = false;
        for (ln, raw) in &lines {
            if *ln >= start && *ln < end {
                let t = raw.trim();
                if t.is_empty() || vol_lines.contains(ln) || chap_lines.contains(ln) || book_re.is_match(t) {
                    continue;
                }
                has_intro = true;
                break;
            }
        }
        if has_intro {
            intro_count += 1;
        }
    }
    let requested = req
        .structure
        .as_deref()
        .map(|s| StructureType::from_str(s).unwrap_or(StructureType::Auto))
        .unwrap_or(StructureType::Auto);
    let auto_detected = if vol_count > 0 && intro_count > 0 {
        StructureType::VolumeIntro
    } else if vol_count > 0 {
        StructureType::VolumeChapters
    } else {
        StructureType::PureChapters
    };
    let effective = if requested == StructureType::Auto {
        auto_detected
    } else {
        requested
    };

    // 生成重整映射：卷 1..n，章按 effective/is_reset
    let mut vol_new_raw: HashMap<u32, String> = HashMap::new();
    for (idx, vol) in raw_volumes.iter().enumerate() {
        let new_num = (idx as i32) + 1;
        let new_raw = format_number(new_num, &vol.num_str);
        vol_new_raw.insert(vol.line_no, new_raw);
    }
    let mut chap_new_raw: HashMap<u32, String> = HashMap::new();
    match effective {
        StructureType::PureChapters => {
            let mut chaps_sorted = raw_chapters.clone();
            chaps_sorted.sort_by_key(|c| c.line_no);
            for (i, c) in chaps_sorted.iter().enumerate() {
                let new_num = (i as i32) + 1;
                chap_new_raw.insert(c.line_no, format_number(new_num, &c.num_str));
            }
        }
        StructureType::VolumeChapters | StructureType::VolumeIntro => {
            // 判定重置 vs 连续（同 inspect）
            let mut first_nums: Vec<Option<i32>> = Vec::new();
            for (i, _) in raw_volumes.iter().enumerate() {
                let first = raw_chapters.iter().find(|c| c.volume_idx == Some(i));
                first_nums.push(first.and_then(|c| c.num));
            }
            let is_reset = if raw_volumes.len() <= 1 {
                true
            } else {
                let mut all_one = true;
                let mut has_chapter_vols = 0;
                for v in first_nums.iter().flatten() {
                    has_chapter_vols += 1;
                    if *v != 1 {
                        all_one = false;
                    }
                }
                if has_chapter_vols == 0 { true } else { all_one }
            };
            if is_reset {
                for (vi, _) in raw_volumes.iter().enumerate() {
                    let mut chaps: Vec<&RawChapter> = raw_chapters.iter().filter(|c| c.volume_idx == Some(vi)).collect();
                    chaps.sort_by_key(|c| c.line_no);
                    for (i, c) in chaps.iter().enumerate() {
                        let new_num = (i as i32) + 1;
                        chap_new_raw.insert(c.line_no, format_number(new_num, &c.num_str));
                    }
                }
                // 卷外章节（无卷）按全局追加（若存在）
                let mut outside: Vec<&RawChapter> = raw_chapters.iter().filter(|c| c.volume_idx.is_none()).collect();
                outside.sort_by_key(|c| c.line_no);
                for (i, c) in outside.iter().enumerate() {
                    // 卷外章节接在最后，编号继续全局？此处按出现顺序独立 1..k
                    let new_num = (i as i32) + 1;
                    chap_new_raw.insert(c.line_no, format_number(new_num, &c.num_str));
                }
            } else {
                let mut chaps_sorted = raw_chapters.clone();
                chaps_sorted.sort_by_key(|c| c.line_no);
                for (i, c) in chaps_sorted.iter().enumerate() {
                    let new_num = (i as i32) + 1;
                    chap_new_raw.insert(c.line_no, format_number(new_num, &c.num_str));
                }
            }
        }
        StructureType::Auto => unreachable!(),
    }

    // 行级重写：仅替换编号捕获组，保留标题与前后空白/标点
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len());
    for (line_no, raw) in &lines {
        let mut new_raw_line = raw.clone();
        if let Some(new_num) = vol_new_raw.get(line_no)
            && let Some(caps) = vol_re.captures(raw)
            && let Some(m) = caps.get(1)
        {
            new_raw_line.replace_range(m.range(), new_num);
        } else if let Some(new_num) = chap_new_raw.get(line_no)
            && let Some(caps) = chap_re.captures(raw)
            && let Some(m) = caps.get(1)
        {
            new_raw_line.replace_range(m.range(), new_num);
        }
        new_lines.push(new_raw_line);
    }
    let new_text = new_lines.join("\n");
    fs::write(&content_path, new_text).map_err(|e| AppError::new(CODE_ERROR, format!("[inspect] 写入重整文件失败: {e}")))?;

    // 更新 metadata modified
    {
        let uuid = sanitize_identifier(&identifier)?;
        let base = app
            .path()
            .resolve(PROJECTS_DIR, BaseDirectory::AppData)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[inspect] resolve dir 失败: {e}")))?;
        let meta_path = base.join(&uuid).join("metadata.json");
        if meta_path.exists()
            && let Ok(data) = fs::read_to_string(&meta_path)
            && let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&data)
        {
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

    // 重扫返回新结果
    let new_req = InspectRequest {
        identifier,
        structure: req.structure,
        volume_regex: req.volume_regex,
        chapter_regex: req.chapter_regex,
    };
    inspect_content(app, new_req)
}

#[cfg(test)]
mod tests {
    use super::parse_chapter_number;
    #[test]
    fn test_cn() {
        assert_eq!(parse_chapter_number("一"), Some(1));
        assert_eq!(parse_chapter_number("十二"), Some(12));
        assert_eq!(parse_chapter_number("二十"), Some(20));
        assert_eq!(parse_chapter_number("二十三"), Some(23));
        assert_eq!(parse_chapter_number("一百零三"), Some(103));
        assert_eq!(parse_chapter_number("一百"), Some(100));
        assert_eq!(parse_chapter_number("十"), Some(10));
        assert_eq!(parse_chapter_number("三千二百一十"), Some(3210));
        assert_eq!(parse_chapter_number("两千"), Some(2000));
        assert_eq!(parse_chapter_number("1"), Some(1));
        assert_eq!(parse_chapter_number("12"), Some(12));
    }
}
