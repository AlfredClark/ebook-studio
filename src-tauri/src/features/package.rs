//! 打包功能：基于 build 未压缩 EPUB 目录生成 outputs 三件套并校验
//!
//! 真相源：`APPDATA/Projects/<uuid>/build/<书名>/`（build 阶段未压缩目录）
//!           + `APPDATA/Projects/<uuid>/split.json` + `metadata.json` + `sources/cover.*`
//! 产物：`APPDATA/Projects/<uuid>/outputs/${sanitize_title}.epub`（EPUB3.3 压缩）
//!      + `outputs/${sanitize_title}.txt`（含 metadata 头的纯文本）
//!      + `outputs/${sanitize_title}.${ext}`（封面拷贝，缺失则 2 文件）
//! 校验：经 `epubveri::validate_path`（EPUB3.3，208/210 epubcheck 规则）产 `Report{messages, epub_version}`

#![allow(clippy::collapsible_if)]

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::cores::response::{AppError, AppResult, CODE_ERROR};
use crate::features::build::{FormatConfig, apply_title_format, compute_pad_width, format_number_display};
use crate::features::split::SplitResult;

const PROJECTS_DIR: &str = "Projects";
const SPLIT_NAME: &str = "split.json";
const METADATA_FILE: &str = "metadata.json";
const BUILD_DIR: &str = "build";
const OUTPUTS_DIR: &str = "outputs";
const FORMAT_NAME: &str = "format.json";

// --- structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageFile {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageIssue {
    pub id: String,
    pub severity: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageResult {
    /// 绝对路径：outputs/<书名>.epub
    pub epub_path: String,
    /// 绝对路径：outputs/<书名>.txt
    pub txt_path: String,
    /// 绝对路径：outputs/<书名>.<ext>，无封面则 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_path: Option<String>,
    /// 三文件清单（按文件名排序）
    pub files: Vec<PackageFile>,
    /// 书名（原始，未 sanitize）
    pub book_title: String,
    /// 校验是否通过（无 Error/Fatal）
    pub verified: bool,
    /// 校验问题（epubveri messages 映射）
    pub issues: Vec<PackageIssue>,
    /// 探测到的 EPUB 版本（如 "3.0"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epub_version: Option<String>,
}

// --- helpers ---

fn sanitize_identifier(identifier: &str) -> AppResult<String> {
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    uuid::Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[package] 非法 identifier"))?;
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[package] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn sanitize_title(title: &str) -> String {
    let mut s = title.trim().to_string();
    let invalid = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for ch in invalid {
        s = s.replace(ch, "_");
    }
    s = s.trim().trim_matches('.').to_string();
    if s.chars().count() > 100 {
        s = s.chars().take(100).collect();
        s = s.trim().to_string();
    }
    if s.is_empty() {
        s = "untitled".to_string();
    }
    s
}

fn resolve_project_base(app: &AppHandle, identifier: &str) -> AppResult<(PathBuf, String)> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[package] resolve Projects dir 失败: {e}")))?;
    Ok((base.join(&uuid), uuid))
}

fn find_book_root(project_dir: &Path) -> AppResult<Option<PathBuf>> {
    let build_base = project_dir.join(BUILD_DIR);
    if !build_base.exists() {
        return Ok(None);
    }
    if let Ok(entries) = fs::read_dir(&build_base) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("mimetype").exists() {
                return Ok(Some(p));
            }
        }
    }
    if build_base.join("mimetype").exists() {
        return Ok(Some(build_base));
    }
    Ok(None)
}

fn collect_output_files(outputs_dir: &Path) -> AppResult<Vec<PackageFile>> {
    let mut out = Vec::new();
    let entries =
        fs::read_dir(outputs_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 outputs 失败: {e}")))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_file() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let meta = fs::metadata(&p).map_err(|e| AppError::new(CODE_ERROR, format!("[package] stat 失败: {e}")))?;
            out.push(PackageFile {
                name: name.clone(),
                path: p.to_string_lossy().to_string(),
                size: meta.len(),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn verify_epub(epub_path: &Path) -> (bool, Vec<PackageIssue>, Option<String>) {
    match epubveri::validate_path(epub_path) {
        Ok(report) => {
            let verified = report.is_valid();
            let epub_version = report.epub_version.clone();
            let issues = report
                .messages
                .iter()
                .map(|m| PackageIssue {
                    id: m.id.to_string(),
                    severity: m.severity.as_str().to_string(),
                    text: m.text.clone(),
                    location: m.location.clone(),
                    position: m.position.map(|p| format!("{}:{}", p.line, p.column)),
                    rule: m.rule.map(|r| r.to_string()),
                })
                .collect();
            (verified, issues, epub_version)
        }
        Err(e) => {
            let issue = PackageIssue {
                id: "PKG-IO".to_string(),
                severity: "fatal".to_string(),
                text: format!("校验读取失败: {e}"),
                location: None,
                position: None,
                rule: None,
            };
            (false, vec![issue], None)
        }
    }
}

fn read_format_config(project_dir: &Path) -> FormatConfig {
    let path = project_dir.join(FORMAT_NAME);
    if !path.exists() {
        return FormatConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!("[package] 解析 format.json 失败: {e}");
                FormatConfig::default()
            }
        },
        Err(e) => {
            log::warn!("[package] 读取 format.json 失败: {e}");
            FormatConfig::default()
        }
    }
}

fn build_txt_content(meta: &serde_json::Value, split: &SplitResult, title: &str, fmt: &FormatConfig) -> String {
    let mut lines: Vec<String> = Vec::new();
    // header from metadata
    lines.push(format!("书名：{title}"));
    let get_str = |k: &str| {
        meta.get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let creator = get_str("creator").unwrap_or_else(|| "佚名".to_string());
    lines.push(format!("作者：{creator}"));
    if let Some(lang) = get_str("language") {
        lines.push(format!("语言：{lang}"));
    }
    if let Some(v) = get_str("creator") {
        if v != creator {
            lines.push(format!("作者：{v}"));
        }
    }
    if let Some(v) = get_str("contributor") {
        lines.push(format!("贡献者：{v}"));
    }
    if let Some(v) = get_str("publisher") {
        lines.push(format!("出版者：{v}"));
    }
    if let Some(v) = get_str("date") {
        lines.push(format!("日期：{v}"));
    }
    if let Some(v) = get_str("rights") {
        lines.push(format!("权利：{v}"));
    }
    if let Some(v) = get_str("source") {
        lines.push(format!("来源：{v}"));
    }
    if let Some(v) = get_str("relation") {
        lines.push(format!("关系：{v}"));
    }
    if let Some(v) = get_str("coverage") {
        lines.push(format!("覆盖范围：{v}"));
    }
    // subjects / description arrays
    let subjects: Vec<String> = meta
        .get("subjects")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !subjects.is_empty() {
        lines.push(format!("标签：{}", subjects.join(" / ")));
    }
    let descriptions: Vec<String> = meta
        .get("description")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !descriptions.is_empty() {
        lines.push("简介：".to_string());
        for d in descriptions {
            lines.push(d);
        }
    }
    lines.push(String::new());
    lines.push("――――――――――――――――".to_string());
    lines.push(String::new());

    // 计算编号补零宽度（与 build 保持一致）
    let total_volumes = split.volumes.as_ref().map(|v| v.len()).unwrap_or(0);
    let total_chapters = if split.type_ == "pure_chapters" {
        split.chapters.as_ref().map(|chs| chs.len()).unwrap_or(0)
    } else {
        split
            .volumes
            .as_ref()
            .map(|vols| vols.iter().map(|v| v.chapters.len()).sum())
            .unwrap_or(0)
    };
    let chap_pad = compute_pad_width(total_chapters);
    let vol_pad = compute_pad_width(total_volumes);

    // body from split（带卷章编号格式化）
    if split.type_ == "pure_chapters" {
        if let Some(chs) = &split.chapters {
            for ch in chs {
                let order_str = format_number_display(ch.order, &fmt.number_format, chap_pad);
                let display = apply_title_format(&fmt.chapter_title_format, &order_str, &ch.title);
                lines.push(display);
                lines.push(String::new());
                for l in &ch.contents {
                    lines.push(l.clone());
                }
                lines.push(String::new());
            }
        }
    } else if let Some(vols) = &split.volumes {
        for vol in vols {
            let vol_order_str = format_number_display(vol.order, &fmt.number_format, vol_pad);
            let vol_display = apply_title_format(&fmt.volume_title_format, &vol_order_str, &vol.title);
            lines.push(vol_display);
            lines.push(String::new());
            if let Some(intro) = &vol.intro {
                for l in intro {
                    lines.push(l.clone());
                }
                if !intro.is_empty() {
                    lines.push(String::new());
                }
            }
            for ch in &vol.chapters {
                let chap_order_str = format_number_display(ch.order, &fmt.number_format, chap_pad);
                let chap_display = apply_title_format(&fmt.chapter_title_format, &chap_order_str, &ch.title);
                lines.push(chap_display);
                lines.push(String::new());
                for l in &ch.contents {
                    lines.push(l.clone());
                }
                lines.push(String::new());
            }
        }
    }
    // trim trailing empty lines but keep at least one newline at end
    while lines.len() > 2 && lines.last().map(|s| s.is_empty()).unwrap_or(false) && lines[lines.len() - 2].is_empty() {
        lines.pop();
    }
    lines.join("\n")
}

fn collect_build_entries(root: &Path) -> AppResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 build 目录失败: {e}")))?;
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// 查询已打包的产物（若 outputs 不存在或无 epub 则 None）
pub(crate) fn get_package(app: &AppHandle, identifier: &str) -> AppResult<Option<PackageResult>> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] identifier 不能为空"));
    }
    let (project_dir, _uuid) = resolve_project_base(app, &id)?;
    let meta_path = project_dir.join(METADATA_FILE);
    if !meta_path.exists() {
        return Ok(None);
    }
    let meta_data =
        fs::read_to_string(&meta_path).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 metadata 失败: {e}")))?;
    let meta: serde_json::Value = serde_json::from_str(&meta_data)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 解析 metadata 失败: {e}")))?;
    let title = meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("untitled")
        .trim()
        .to_string();
    let sanitized = sanitize_title(&title);
    let outputs_dir = project_dir.join(OUTPUTS_DIR);
    if !outputs_dir.exists() {
        return Ok(None);
    }
    let epub_path = outputs_dir.join(format!("{sanitized}.epub"));
    if !epub_path.exists() {
        return Ok(None);
    }
    let txt_path = outputs_dir.join(format!("{sanitized}.txt"));
    // cover: outputs/<sanitized>.<ext> — scan for sanitized.* != epub/txt
    let mut cover_path_opt: Option<PathBuf> = None;
    if let Ok(entries) = fs::read_dir(&outputs_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_file() {
                let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if fname == format!("{sanitized}.epub") || fname == format!("{sanitized}.txt") {
                    continue;
                }
                if fname.starts_with(&sanitized) && p.extension().is_some() {
                    cover_path_opt = Some(p);
                    break;
                }
            }
        }
    }
    let files = collect_output_files(&outputs_dir)?;
    Ok(Some(PackageResult {
        epub_path: epub_path.to_string_lossy().to_string(),
        txt_path: txt_path.to_string_lossy().to_string(),
        cover_path: cover_path_opt.map(|p| p.to_string_lossy().to_string()),
        files,
        book_title: title,
        verified: false,
        issues: Vec::new(),
        epub_version: None,
    }))
}

/// 校验已打包的 EPUB（按需触发，避免页面加载卡顿）
pub(crate) fn verify_package(app: &AppHandle, identifier: &str) -> AppResult<PackageResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] identifier 不能为空"));
    }
    let (project_dir, _uuid) = resolve_project_base(app, &id)?;
    let meta_path = project_dir.join(METADATA_FILE);
    if !meta_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[package] metadata.json 不存在"));
    }
    let meta_data =
        fs::read_to_string(&meta_path).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 metadata 失败: {e}")))?;
    let meta: serde_json::Value = serde_json::from_str(&meta_data)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 解析 metadata 失败: {e}")))?;
    let title = meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("untitled")
        .trim()
        .to_string();
    let sanitized = sanitize_title(&title);
    let outputs_dir = project_dir.join(OUTPUTS_DIR);
    if !outputs_dir.exists() {
        return Err(AppError::new(CODE_ERROR, "[package] 输出目录不存在"));
    }
    let epub_path = outputs_dir.join(format!("{sanitized}.epub"));
    if !epub_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[package] EPUB 不存在，请先打包"));
    }
    let txt_path = outputs_dir.join(format!("{sanitized}.txt"));
    let mut cover_path_opt: Option<PathBuf> = None;
    if let Ok(entries) = fs::read_dir(&outputs_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_file() {
                let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if fname == format!("{sanitized}.epub") || fname == format!("{sanitized}.txt") {
                    continue;
                }
                if fname.starts_with(&sanitized) && p.extension().is_some() {
                    cover_path_opt = Some(p);
                    break;
                }
            }
        }
    }
    let files = collect_output_files(&outputs_dir)?;
    let (verified, issues, epub_version) = verify_epub(&epub_path);
    Ok(PackageResult {
        epub_path: epub_path.to_string_lossy().to_string(),
        txt_path: txt_path.to_string_lossy().to_string(),
        cover_path: cover_path_opt.map(|p| p.to_string_lossy().to_string()),
        files,
        book_title: title,
        verified,
        issues,
        epub_version,
    })
}

/// 删除 outputs 目录
pub(crate) fn remove_package(app: &AppHandle, identifier: &str) -> AppResult<()> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] identifier 不能为空"));
    }
    let (project_dir, _uuid) = resolve_project_base(app, &id)?;
    let outputs_dir = project_dir.join(OUTPUTS_DIR);
    if outputs_dir.exists() {
        fs::remove_dir_all(&outputs_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 删除 outputs 失败: {e}")))?;
        log::info!("[package] removed {} for {}", outputs_dir.display(), id);
    }
    Ok(())
}

/// 执行打包：压缩 build 目录为 epub + 生成 txt + 拷贝封面 + 校验
pub(crate) fn package_epub(app: &AppHandle, identifier: &str) -> AppResult<PackageResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] identifier 不能为空"));
    }
    let (project_dir, _uuid) = resolve_project_base(app, &id)?;
    let meta_path = project_dir.join(METADATA_FILE);
    if !meta_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[package] metadata.json 不存在"));
    }
    let meta_data =
        fs::read_to_string(&meta_path).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 metadata 失败: {e}")))?;
    let meta: serde_json::Value = serde_json::from_str(&meta_data)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 解析 metadata 失败: {e}")))?;
    let title = meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("untitled")
        .trim()
        .to_string();
    if title.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] 书名不能为空"));
    }
    let sanitized = sanitize_title(&title);

    let split_path = project_dir.join(SPLIT_NAME);
    if !split_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[package] split.json 不存在，请先完成拆分"));
    }
    let split_data =
        fs::read_to_string(&split_path).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 split 失败: {e}")))?;
    let split: SplitResult =
        serde_json::from_str(&split_data).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 解析 split 失败: {e}")))?;

    let Some(book_root) = find_book_root(&project_dir)? else {
        return Err(AppError::new(CODE_ERROR, "[package] 尚未构建，请先完成构建"));
    };

    // 准备 outputs：清空重建（前端已 ConfirmDialog，此处直接覆盖）
    let outputs_dir = project_dir.join(OUTPUTS_DIR);
    if outputs_dir.exists() {
        fs::remove_dir_all(&outputs_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 清理 outputs 失败: {e}")))?;
    }
    fs::create_dir_all(&outputs_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 创建 outputs 失败: {e}")))?;

    // 1. 生成 txt（含 metadata 头，卷章编号按构建时的 format.json）
    let fmt_cfg = read_format_config(&project_dir);
    let txt_content = build_txt_content(&meta, &split, &title, &fmt_cfg);
    let txt_path = outputs_dir.join(format!("{sanitized}.txt"));
    fs::write(&txt_path, txt_content).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 写入 txt 失败: {e}")))?;

    // 2. 拷贝封面 → outputs/<书名>.<ext>
    let mut cover_path_opt: Option<PathBuf> = None;
    if let Some(rel) = meta.get("cover").and_then(|v| v.as_str()).map(|s| s.to_string()) {
        let src = project_dir.join(&rel);
        if src.exists() && src.is_file() {
            if let Some(ext) = src.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
                let dest = outputs_dir.join(format!("{sanitized}.{ext}"));
                fs::copy(&src, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 复制封面失败: {e}")))?;
                cover_path_opt = Some(dest);
            }
        }
    }

    // 3. 压缩 epub：EPUB3.3 mimetype 首条 Stored，其余 Deflated
    let epub_path = outputs_dir.join(format!("{sanitized}.epub"));
    let epub_file =
        File::create(&epub_path).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 创建 epub 失败: {e}")))?;
    let mut zip = zip::ZipWriter::new(epub_file);

    // mimetype 必须首条、Stored、无 extra
    let mimetype_src = book_root.join("mimetype");
    let mimetype_bytes =
        fs::read(&mimetype_src).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 mimetype 失败: {e}")))?;
    if mimetype_bytes != b"application/epub+zip" {
        log::warn!("[package] mimetype 非标准，仍按 Stored 写入");
    }
    {
        use zip::write::SimpleFileOptions;
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", opts)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[package] zip mimetype 失败: {e}")))?;
        zip.write_all(&mimetype_bytes)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 写入 mimetype 失败: {e}")))?;
    }

    // 收集其余文件（排除 mimetype），按相对路径排序
    let all_entries = collect_build_entries(&book_root)?;
    let mut rel_files: Vec<(String, PathBuf)> = Vec::new();
    for abs in all_entries {
        let rel = abs
            .strip_prefix(&book_root)
            .unwrap_or(&abs)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        if rel == "mimetype" {
            continue;
        }
        rel_files.push((rel, abs));
    }
    rel_files.sort_by(|a, b| a.0.cmp(&b.0));
    // 确保 META-INF/container.xml 紧随 mimetype（按 EPUB 常见约定排序，已满足字典序）
    for (rel, abs) in rel_files {
        // 目录项已隐式，无需显式 add_directory（但若 rel 含 /，ZipWriter 会自动处理）
        let mut f = File::open(&abs).map_err(|e| AppError::new(CODE_ERROR, format!("[package] 打开 {rel} 失败: {e}")))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 读取 {rel} 失败: {e}")))?;
        let is_text = matches!(
            Path::new(&rel)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default()
                .as_str(),
            "xhtml" | "html" | "xml" | "opf" | "css" | "txt" | "json" | "ncx"
        );
        // 文本与二进制均 Deflated（epub 允许全 Deflated 除 mimetype）
        let _ = is_text;
        use zip::write::SimpleFileOptions;
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(&rel, opts)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[package] zip entry {rel} 失败: {e}")))?;
        zip.write_all(&buf)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 写入 {rel} 失败: {e}")))?;
    }

    zip.finish()
        .map_err(|e| AppError::new(CODE_ERROR, format!("[package] 完成 zip 失败: {e}")))?;

    // 4. 校验
    let (verified, issues, epub_version) = verify_epub(&epub_path);

    let files = collect_output_files(&outputs_dir)?;
    let result = PackageResult {
        epub_path: epub_path.to_string_lossy().to_string(),
        txt_path: txt_path.to_string_lossy().to_string(),
        cover_path: cover_path_opt.map(|p| p.to_string_lossy().to_string()),
        files,
        book_title: title.clone(),
        verified,
        issues,
        epub_version,
    };
    log::info!("[package] packaged {} -> {} verified={}", id, epub_path.display(), verified);
    Ok(result)
}

/// 获取 outputs 目录绝对路径（供 opener 揭示）
pub(crate) fn get_package_path(app: &AppHandle, identifier: &str) -> AppResult<Option<String>> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[package] identifier 不能为空"));
    }
    let (project_dir, _uuid) = resolve_project_base(app, &id)?;
    let outputs_dir = project_dir.join(OUTPUTS_DIR);
    if !outputs_dir.exists() {
        return Ok(None);
    }
    // 即使空也返回路径，前端由 revealItemInDir 处理
    Ok(Some(outputs_dir.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{build_txt_content, sanitize_title};
    use crate::features::{
        build::FormatConfig,
        split::{SplitChapter, SplitResult, SplitVolume},
    };
    use serde_json::json;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_title(" a/b:c*? "), "a_b_c__");
        assert_eq!(sanitize_title(""), "untitled");
    }

    #[test]
    fn test_txt_pure() {
        let meta = json!({
            "title": "测试书",
            "creator": "作者A",
            "language": "zh-CN",
            "description": ["简介一行"],
            "subjects": ["玄幻","修真"]
        });
        let split = SplitResult {
            type_: "pure_chapters".to_string(),
            volumes: None,
            chapters: Some(vec![
                SplitChapter {
                    order: 1,
                    title: "风起".to_string(),
                    contents: vec!["正文1".to_string()],
                },
                SplitChapter {
                    order: 2,
                    title: "云涌".to_string(),
                    contents: vec!["正文2".to_string()],
                },
            ]),
        };
        let fmt = FormatConfig::default();
        let txt = build_txt_content(&meta, &split, "测试书", &fmt);
        assert!(txt.contains("书名：测试书"));
        assert!(txt.contains("作者：作者A"));
        assert!(txt.contains("标签：玄幻 / 修真"));
        // 编号格式化：默认 第{order}章 {title} + arabic
        assert!(txt.contains("第1章 风起"));
        assert!(txt.contains("第2章 云涌"));
        assert!(txt.contains("正文1"));
    }

    #[test]
    fn test_txt_volume() {
        let meta = json!({"title": "卷书"});
        let split = SplitResult {
            type_: "volume_chapters".to_string(),
            volumes: Some(vec![SplitVolume {
                order: 1,
                title: "卷头".to_string(),
                intro: Some(vec!["卷简介".to_string()]),
                chapters: vec![SplitChapter {
                    order: 1,
                    title: "序章".to_string(),
                    contents: vec!["c1".to_string()],
                }],
            }]),
            chapters: None,
        };
        let fmt = FormatConfig::default();
        let txt = build_txt_content(&meta, &split, "卷书", &fmt);
        assert!(txt.contains("第1卷 卷头"));
        assert!(txt.contains("卷简介"));
        assert!(txt.contains("第1章 序章"));
    }
}
