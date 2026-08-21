//! 构建功能：基于 split.json + metadata.json 生成未压缩 EPUB 目录
//!
//! 真相源：`APPDATA/Projects/<uuid>/split.json + metadata.json + sources/cover.*`
//! 产物：`APPDATA/Projects/<uuid>/build/<书名>/`（mimetype/META-INF/EPUB）

#![allow(clippy::collapsible_if)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::cores::response::{AppError, AppResult, CODE_ERROR};
use crate::features::split::{SplitResult, SplitVolume};

const PROJECTS_DIR: &str = "Projects";
const SPLIT_NAME: &str = "split.json";
const METADATA_FILE: &str = "metadata.json";
const BUILD_DIR: &str = "build";

const DEFAULT_CHAPTER_TITLE_FORMAT: &str = "第{order}章 {title}";
const DEFAULT_VOLUME_TITLE_FORMAT: &str = "第{order}卷 {title}";
const FORMAT_NAME: &str = "format.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatConfig {
    pub chapter_title_format: String,
    pub volume_title_format: String,
    pub number_format: String,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            chapter_title_format: DEFAULT_CHAPTER_TITLE_FORMAT.to_string(),
            volume_title_format: DEFAULT_VOLUME_TITLE_FORMAT.to_string(),
            number_format: "arabic".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildFile {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<BuildFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    /// 绝对路径：build/<书名>
    pub epub_path: String,
    /// 目录树（以书名根为起点）
    pub files: Vec<BuildFile>,
    /// 书名（用于展示）
    pub book_title: String,
}

// --- helpers ---
fn sanitize_identifier(identifier: &str) -> AppResult<String> {
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    uuid::Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[build] 非法 identifier"))?;
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[build] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn sanitize_title(title: &str) -> String {
    let mut s = title.trim().to_string();
    // 非法字符：/ \ : * ? " < > |  + 控制字符
    let invalid = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for ch in invalid {
        s = s.replace(ch, "_");
    }
    // 去除前后空格点
    s = s.trim().trim_matches('.').to_string();
    // 限制长度 100 字符
    if s.chars().count() > 100 {
        s = s.chars().take(100).collect();
        s = s.trim().to_string();
    }
    if s.is_empty() {
        s = "untitled".to_string();
    }
    s
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn media_type_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "image/jpeg",
    }
}

fn is_text_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "xhtml" | "html" | "xml" | "opf" | "css" | "txt" | "json")
}

fn resolve_project_base(app: &AppHandle, identifier: &str) -> AppResult<(PathBuf, String, String)> {
    let uuid = sanitize_identifier(identifier)?;
    let base = app
        .path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] resolve Projects dir 失败: {e}")))?;
    let project_dir = base.join(&uuid);
    Ok((project_dir, uuid, base.to_string_lossy().to_string()))
}

fn read_template(app: &AppHandle, rel: &str) -> AppResult<String> {
    // 1) resource_dir/templates/...
    if let Ok(res_dir) = app.path().resource_dir() {
        let p = res_dir.join("templates").join(rel);
        if p.exists() {
            if let Ok(s) = fs::read_to_string(&p) {
                return Ok(s);
            }
        }
        // also try templates/EPUB33-NOVEL variant
        let p2 = res_dir.join(rel);
        if p2.exists() {
            if let Ok(s) = fs::read_to_string(&p2) {
                return Ok(s);
            }
        }
    }
    // 2) dev fallback: src-tauri/templates/...
    let candidates = [
        PathBuf::from(format!("src-tauri/templates/{rel}")),
        PathBuf::from(format!("templates/{rel}")),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join(rel),
    ];
    for p in candidates {
        if p.exists() {
            if let Ok(s) = fs::read_to_string(&p) {
                return Ok(s);
            }
        }
    }
    Err(AppError::new(CODE_ERROR, format!("[build] 模板缺失: {rel}")))
}

#[allow(dead_code)]
fn read_template_bytes(app: &AppHandle, rel: &str) -> AppResult<Vec<u8>> {
    if let Ok(res_dir) = app.path().resource_dir() {
        let p = res_dir.join("templates").join(rel);
        if p.exists() {
            if let Ok(b) = fs::read(&p) {
                return Ok(b);
            }
        }
    }
    let candidates = [
        PathBuf::from(format!("src-tauri/templates/{rel}")),
        PathBuf::from(format!("templates/{rel}")),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join(rel),
    ];
    for p in candidates {
        if p.exists() {
            if let Ok(b) = fs::read(&p) {
                return Ok(b);
            }
        }
    }
    Err(AppError::new(CODE_ERROR, format!("[build] 模板缺失: {rel}")))
}

fn build_file_tree(root: &Path, base: &Path) -> AppResult<Vec<BuildFile>> {
    let mut out = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 读取目录失败: {e}")))?;
    let mut items: Vec<_> = entries.flatten().collect();
    items.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        if a_is_dir != b_is_dir {
            return b_is_dir.cmp(&a_is_dir); // dirs first
        }
        a.file_name().cmp(&b.file_name())
    });
    for entry in items {
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let is_dir = path.is_dir();
        let children = if is_dir { Some(build_file_tree(&path, base)?) } else { None };
        out.push(BuildFile {
            path: rel,
            name,
            is_dir,
            children,
        });
    }
    Ok(out)
}

fn collect_build_files(root: &Path, base: &Path) -> AppResult<Vec<BuildFile>> {
    build_file_tree(root, base)
}

pub(crate) fn format_number_display(order: i32, number_format: &str, pad_width: usize) -> String {
    match number_format {
        "arabic" => order.to_string(),
        "arabic_padded" => format!("{order:0>width$}", width = pad_width),
        "chinese_lower" => {
            use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
            match order.to_chinese(ChineseVariant::Simple, ChineseCase::Lower, ChineseCountMethod::TenThousand) {
                Ok(s) => s,
                Err(_) => order.to_string(),
            }
        }
        "chinese_upper" => {
            use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
            // 尝试繁体大写，若失败回退简体大写
            match order.to_chinese(ChineseVariant::Simple, ChineseCase::Upper, ChineseCountMethod::TenThousand) {
                Ok(s) => s,
                Err(_) => order.to_string(),
            }
        }
        _ => order.to_string(),
    }
}

pub(crate) fn apply_title_format(format: &str, order_str: &str, title: &str) -> String {
    // 必须包含 {title}，若不含则直接返回标题；{order} 可选
    let mut out = format.to_string();
    if format.contains("{order}") {
        out = out.replace("{order}", order_str);
    }
    if format.contains("{title}") {
        out = out.replace("{title}", title);
    } else {
        // 若模板未包含 {title}，则追加标题
        if !out.contains(title) {
            out = format!("{out} {title}");
        }
    }
    out.trim().to_string()
}

pub(crate) fn compute_pad_width(count: usize) -> usize {
    count.to_string().len().max(2)
}

/// 读取已保存的格式化配置（format.json），不存在或损坏则返回 None
pub(crate) fn get_format(app: &AppHandle, identifier: &str) -> AppResult<Option<FormatConfig>> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let (project_dir, _uuid, _) = resolve_project_base(app, &id)?;
    let path = project_dir.join(FORMAT_NAME);
    if !path.exists() {
        return Ok(None);
    }
    let data =
        fs::read_to_string(&path).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 读取 format.json 失败: {e}")))?;
    let cfg: FormatConfig = serde_json::from_str(&data).map_err(|e| {
        log::warn!("[build] 解析 format.json 失败: {e}");
        AppError::new(CODE_ERROR, format!("[build] 解析 format.json 失败: {e}"))
    })?;
    Ok(Some(cfg))
}

/// 查询已构建的 EPUB 目录（若不存在返回 None）
pub(crate) fn get_build(app: &AppHandle, identifier: &str) -> AppResult<Option<BuildResult>> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let (project_dir, _uuid, _) = resolve_project_base(app, &id)?;
    let build_base = project_dir.join(BUILD_DIR);
    if !build_base.exists() {
        return Ok(None);
    }
    // 寻找 build/<书名>/mimetype
    let mut found: Option<PathBuf> = None;
    let mut book_title = String::new();
    if let Ok(entries) = fs::read_dir(&build_base) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let mimetype = p.join("mimetype");
                if mimetype.exists() {
                    found = Some(p.clone());
                    book_title = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    break;
                }
                // 兼容旧结构 build/EPUB
                let alt = p.join("EPUB").join("content.opf");
                if alt.exists() {
                    found = Some(p.clone());
                    book_title = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    break;
                }
            }
        }
    }
    // 兼容：若 build_base 本身即为 EPUB 根（直接含 mimetype）
    if found.is_none() && build_base.join("mimetype").exists() {
        found = Some(build_base.clone());
        // 尝试从 metadata 取标题
        let meta_path = project_dir.join(METADATA_FILE);
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(t) = v.get("title").and_then(|x| x.as_str()) {
                    book_title = t.to_string();
                }
            }
        }
    }
    let Some(root) = found else {
        return Ok(None);
    };
    let files = collect_build_files(&root, &root)?;
    Ok(Some(BuildResult {
        epub_path: root.to_string_lossy().to_string(),
        files,
        book_title,
    }))
}

/// 删除已构建的目录（用于重新构建流程）
pub(crate) fn remove_build(app: &AppHandle, identifier: &str) -> AppResult<()> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let (project_dir, _uuid, _) = resolve_project_base(app, &id)?;
    let build_base = project_dir.join(BUILD_DIR);
    if build_base.exists() {
        fs::remove_dir_all(&build_base).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 删除构建目录失败: {e}")))?;
        log::info!("[build] removed {} for {}", build_base.display(), id);
    }
    Ok(())
}

/// 执行构建
pub(crate) fn build_epub(
    app: &AppHandle,
    identifier: &str,
    chapter_title_format: Option<String>,
    volume_title_format: Option<String>,
    number_format: Option<String>,
) -> AppResult<BuildResult> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let (project_dir, uuid, _) = resolve_project_base(app, &id)?;
    let meta_path = project_dir.join(METADATA_FILE);
    if !meta_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[build] metadata.json 不存在"));
    }
    let meta_data =
        fs::read_to_string(&meta_path).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 读取 metadata 失败: {e}")))?;
    let meta: serde_json::Value =
        serde_json::from_str(&meta_data).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 解析 metadata 失败: {e}")))?;

    let title = meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("untitled")
        .trim()
        .to_string();
    if title.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] 书名不能为空"));
    }
    let sanitized_title = sanitize_title(&title);

    // 读取 split
    let split_path = project_dir.join(SPLIT_NAME);
    if !split_path.exists() {
        return Err(AppError::new(CODE_ERROR, "[build] split.json 不存在，请先完成拆分"));
    }
    let split_data =
        fs::read_to_string(&split_path).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 读取 split 失败: {e}")))?;
    let split: SplitResult =
        serde_json::from_str(&split_data).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 解析 split 失败: {e}")))?;

    // 计算补零宽度（自适应，卷与章节分开计算）
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
    let chap_pad_width = compute_pad_width(total_chapters);
    let vol_pad_width = compute_pad_width(total_volumes);

    // 标题与编号格式（带校验）
    let chap_fmt_raw = chapter_title_format
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_CHAPTER_TITLE_FORMAT.to_string());
    let vol_fmt_raw = volume_title_format
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_VOLUME_TITLE_FORMAT.to_string());
    let num_fmt_raw = number_format
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "arabic".to_string());

    // 校验必须含 {title}
    let chap_fmt = if chap_fmt_raw.contains("{title}") {
        chap_fmt_raw.clone()
    } else {
        // 若不含 title 占位，强制追加
        format!("{chap_fmt_raw} {{title}}")
    };
    let vol_fmt = if vol_fmt_raw.contains("{title}") {
        vol_fmt_raw.clone()
    } else {
        format!("{vol_fmt_raw} {{title}}")
    };
    let num_fmt = match num_fmt_raw.as_str() {
        "arabic" | "arabic_padded" | "chinese_lower" | "chinese_upper" => num_fmt_raw.clone(),
        _ => "arabic".to_string(),
    };

    // 持久化格式化配置供 package 阶段复用
    let fmt_cfg = FormatConfig {
        chapter_title_format: chap_fmt.clone(),
        volume_title_format: vol_fmt.clone(),
        number_format: num_fmt.clone(),
    };
    let fmt_path = project_dir.join(FORMAT_NAME);
    if let Err(e) = fs::write(&fmt_path, serde_json::to_string_pretty(&fmt_cfg).unwrap_or_default()) {
        log::warn!("[build] 写入 format.json 失败: {e}");
    }

    // 清理旧 build
    let build_base = project_dir.join(BUILD_DIR);
    if build_base.exists() {
        let _ = fs::remove_dir_all(&build_base);
    }
    fs::create_dir_all(&build_base).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建 build 目录失败: {e}")))?;
    let book_root = build_base.join(&sanitized_title);
    fs::create_dir_all(&book_root).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建书名目录失败: {e}")))?;

    // 提取 metadata 字段
    let language = meta
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("zh-CN")
        .trim()
        .to_string();
    let language_esc = if language.is_empty() {
        "zh-CN".to_string()
    } else {
        language.clone()
    };
    let creator = meta
        .get("creator")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let contributor = meta
        .get("contributor")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let publisher = meta
        .get("publisher")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let date = meta
        .get("date")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let rights = meta
        .get("rights")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let source = meta
        .get("source")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let relation = meta
        .get("relation")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let coverage = meta
        .get("coverage")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
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
    let modified = meta
        .get("modified")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    let book_uuid = uuid.clone();
    let book_author = creator.clone().unwrap_or_else(|| "佚名".to_string());

    // 处理封面
    let cover_rel = meta.get("cover").and_then(|v| v.as_str()).map(|s| s.to_string());
    let mut cover_item = String::new();
    let mut cover_block = String::new();
    let mut cover_ext_opt: Option<String> = None;
    let mut cover_src_abs: Option<PathBuf> = None;
    if let Some(rel) = &cover_rel {
        let src = project_dir.join(rel);
        if src.exists() && src.is_file() {
            if let Some(ext) = src.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
                cover_ext_opt = Some(ext.clone());
                let media = media_type_for_ext(&ext);
                cover_item = format!(
                    r#"<item id="cover-image" href="images/cover.{ext}" media-type="{media}" properties="cover-image"/>"#
                );
                cover_block = format!(r#"<img src="images/cover.{ext}" alt="封面" />"#);
                cover_src_abs = Some(src);
            }
        }
    }
    if cover_item.is_empty() {
        // 无封面 fallback
        cover_block = format!(r#"<h1 class="book-title">{}</h1>"#, escape_xml(&title));
    }

    // 收集章节/卷信息用于 manifest/spine/toc（带补零）
    #[allow(dead_code)]
    #[derive(Clone)]
    struct ChapInfo {
        id: String,
        href: String,
        title: String,
        display_title: String,
        vol_order: Option<i32>,
        chap_order: i32,
    }
    #[allow(dead_code)]
    #[derive(Clone)]
    struct VolInfo {
        id: String,
        href: String,
        title: String,
        display_title: String,
        order: i32,
    }

    let mut vol_infos: Vec<VolInfo> = Vec::new();
    let mut chap_infos: Vec<ChapInfo> = Vec::new();

    if split.type_ == "pure_chapters" {
        if let Some(chs) = &split.chapters {
            for ch in chs {
                let padded = format!("{:0>width$}", ch.order, width = chap_pad_width);
                let formatted_order = format_number_display(ch.order, &num_fmt, chap_pad_width);
                let display_title = apply_title_format(&chap_fmt, &formatted_order, &ch.title);
                // 为了展示需要保留 padded 与 formatted 区别：文件名用 padded 阿拉伯，标题用 formatted
                // 但若 number_format 本身为 arabic_padded，则两者一致；否则文件名用 padded，标题用 formatted
                let file_padded = if num_fmt == "arabic_padded" {
                    formatted_order.clone()
                } else {
                    padded.clone()
                };
                let id = format!("chapter-{file_padded}");
                let href = format!("text/{id}.xhtml");
                chap_infos.push(ChapInfo {
                    id: id.clone(),
                    href: href.clone(),
                    title: ch.title.clone(),
                    display_title,
                    vol_order: None,
                    chap_order: ch.order,
                });
                let _ = padded;
                let _ = file_padded;
            }
        }
    } else if let Some(vols) = &split.volumes {
        for vol in vols {
            let padded_vol = format!("{:0>width$}", vol.order, width = vol_pad_width);
            let formatted_vol_order = format_number_display(vol.order, &num_fmt, vol_pad_width);
            let display_vol_title = apply_title_format(&vol_fmt, &formatted_vol_order, &vol.title);
            let vol_id = format!("volume-{padded_vol}");
            let vol_href = format!("text/{vol_id}.xhtml");
            vol_infos.push(VolInfo {
                id: vol_id.clone(),
                href: vol_href,
                title: vol.title.clone(),
                display_title: display_vol_title,
                order: vol.order,
            });
            for ch in &vol.chapters {
                let padded_chap = format!("{:0>width$}", ch.order, width = chap_pad_width);
                let formatted_chap_order = format_number_display(ch.order, &num_fmt, chap_pad_width);
                let display_chap_title = apply_title_format(&chap_fmt, &formatted_chap_order, &ch.title);
                let file_padded_vol = padded_vol.clone();
                let file_padded_chap = if num_fmt == "arabic_padded" {
                    formatted_chap_order.clone()
                } else {
                    padded_chap.clone()
                };
                let chap_id = format!("chapter-{file_padded_vol}-{file_padded_chap}");
                let chap_href = format!("text/{chap_id}.xhtml");
                chap_infos.push(ChapInfo {
                    id: chap_id,
                    href: chap_href,
                    title: ch.title.clone(),
                    display_title: display_chap_title,
                    vol_order: Some(vol.order),
                    chap_order: ch.order,
                });
            }
        }
    }

    // 生成目录结构：mimetype, META-INF, EPUB/styles, EPUB/text, EPUB/images
    let mimetype_path = book_root.join("mimetype");
    fs::write(&mimetype_path, "application/epub+zip")
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 mimetype 失败: {e}")))?;

    let meta_inf_dir = book_root.join("META-INF");
    fs::create_dir_all(&meta_inf_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建 META-INF 失败: {e}")))?;
    let container_src = read_template(app, "EPUB33-NOVEL/META-INF/container.xml")?;
    fs::write(meta_inf_dir.join("container.xml"), container_src)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 container.xml 失败: {e}")))?;

    let epub_dir = book_root.join("EPUB");
    let text_dir = epub_dir.join("text");
    let styles_dir = epub_dir.join("styles");
    let images_dir = epub_dir.join("images");
    fs::create_dir_all(&text_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建 text 失败: {e}")))?;
    fs::create_dir_all(&styles_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建 styles 失败: {e}")))?;
    // styles/base.css
    let css_content = read_template(app, "EPUB33-NOVEL/EPUB/styles/base.css").unwrap_or_else(|_| "".to_string());
    if !css_content.is_empty() {
        fs::write(styles_dir.join("base.css"), css_content)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 base.css 失败: {e}")))?;
    }

    // 复制封面
    if let (Some(src), Some(ext)) = (cover_src_abs, cover_ext_opt) {
        fs::create_dir_all(&images_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建 images 失败: {e}")))?;
        let dest = images_dir.join(format!("cover.{ext}"));
        fs::copy(&src, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 复制封面失败: {e}")))?;
    }

    // 生成 chapter/volume xhtml
    let chapter_tpl = read_template(app, "EPUB33-NOVEL/EPUB/text/chapter-template.xhtml")?;
    let volume_tpl = read_template(app, "EPUB33-NOVEL/EPUB/text/volume-template.xhtml")?;

    // 建立快速查找表
    use std::collections::HashMap;
    let mut chap_map: HashMap<(Option<i32>, i32), &crate::features::split::SplitChapter> = HashMap::new();
    if split.type_ == "pure_chapters" {
        if let Some(chs) = &split.chapters {
            for ch in chs {
                chap_map.insert((None, ch.order), ch);
            }
        }
    } else if let Some(vols) = &split.volumes {
        for vol in vols {
            for ch in &vol.chapters {
                chap_map.insert((Some(vol.order), ch.order), ch);
            }
        }
    }
    let mut vol_map: HashMap<i32, &SplitVolume> = HashMap::new();
    if let Some(vols) = &split.volumes {
        for vol in vols {
            vol_map.insert(vol.order, vol);
        }
    }

    for chap in &chap_infos {
        let key = (chap.vol_order, chap.chap_order);
        let Some(src_chap) = chap_map.get(&key) else { continue };
        let body = src_chap
            .contents
            .iter()
            .map(|l| format!("<p>{}</p>", escape_xml(l)))
            .collect::<Vec<_>>()
            .join("\n");
        let rendered = chapter_tpl
            .replace("{{CHAPTER_TITLE}}", &escape_xml(&chap.display_title))
            .replace("{{CHAPTER_BODY}}", &body);
        fs::write(text_dir.join(format!("{}.xhtml", chap.id)), rendered)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 {} 失败: {e}", chap.id)))?;
    }

    for vol in &vol_infos {
        let Some(src_vol) = vol_map.get(&vol.order) else { continue };
        let intro_html = if let Some(intro) = &src_vol.intro {
            if intro.is_empty() {
                "".to_string()
            } else {
                intro
                    .iter()
                    .map(|l| format!(r#"<p class="volume-intro">{}</p>"#, escape_xml(l)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else {
            "".to_string()
        };
        let rendered = volume_tpl
            .replace("{{VOLUME_TITLE}}", &escape_xml(&vol.display_title))
            .replace("{{VOLUME_INTRO}}", &intro_html);
        fs::write(text_dir.join(format!("{}.xhtml", vol.id)), rendered)
            .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 {} 失败: {e}", vol.id)))?;
    }

    // 生成 content.opf
    let mut opf_tpl = read_template(app, "EPUB33-NOVEL/EPUB/content.opf")?;
    // 构造 manifest/items（与 spine/toc 同序交错，避免阅读序不一致）
    let mut manifest_items = String::new();
    if !vol_infos.is_empty() {
        for vol in &vol_infos {
            manifest_items.push_str(&format!(
                r#"    <item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                vol.id, vol.href
            ));
            manifest_items.push('\n');
            for chap in chap_infos.iter().filter(|c| c.vol_order == Some(vol.order)) {
                manifest_items.push_str(&format!(
                    r#"    <item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                    chap.id, chap.href
                ));
                manifest_items.push('\n');
            }
        }
        for chap in chap_infos.iter().filter(|c| c.vol_order.is_none()) {
            manifest_items.push_str(&format!(
                r#"    <item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                chap.id, chap.href
            ));
            manifest_items.push('\n');
        }
    } else {
        for chap in &chap_infos {
            manifest_items.push_str(&format!(
                r#"    <item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                chap.id, chap.href
            ));
            manifest_items.push('\n');
        }
    }
    // 去掉末尾换行
    if manifest_items.ends_with('\n') {
        manifest_items.pop();
    }

    let mut spine_items = String::new();
    if !vol_infos.is_empty() {
        for vol in &vol_infos {
            spine_items.push_str(&format!(r#"    <itemref idref="{}"/>"#, vol.id));
            spine_items.push('\n');
            for chap in chap_infos.iter().filter(|c| c.vol_order == Some(vol.order)) {
                spine_items.push_str(&format!(r#"    <itemref idref="{}"/>"#, chap.id));
                spine_items.push('\n');
            }
        }
        for chap in chap_infos.iter().filter(|c| c.vol_order.is_none()) {
            spine_items.push_str(&format!(r#"    <itemref idref="{}"/>"#, chap.id));
            spine_items.push('\n');
        }
    } else {
        for chap in &chap_infos {
            spine_items.push_str(&format!(r#"    <itemref idref="{}"/>"#, chap.id));
            spine_items.push('\n');
        }
    }
    if spine_items.ends_with('\n') {
        spine_items.pop();
    }

    // 统一其他元数据块：description（单标签 &#10;）+ subjects + publisher/date/rights/source/relation/coverage，按块空行分隔
    let mut other_blocks: Vec<String> = Vec::new();
    if !descriptions.is_empty() {
        let joined = descriptions
            .iter()
            .map(|d| escape_xml(d.trim()))
            .collect::<Vec<_>>()
            .join("&#10;");
        other_blocks.push(format!("    <dc:description>{joined}</dc:description>"));
    }
    if !subjects.is_empty() {
        let subjects_block = subjects
            .iter()
            .map(|s| format!("    <dc:subject>{}</dc:subject>", escape_xml(s)))
            .collect::<Vec<_>>()
            .join("\n");
        other_blocks.push(subjects_block);
    }
    let mut optional_lines: Vec<String> = Vec::new();
    if let Some(p) = &publisher {
        optional_lines.push(format!("    <dc:publisher>{}</dc:publisher>", escape_xml(p)));
    }
    if let Some(d) = &date {
        optional_lines.push(format!("    <dc:date>{}</dc:date>", escape_xml(d)));
    }
    if let Some(r) = &rights {
        optional_lines.push(format!("    <dc:rights>{}</dc:rights>", escape_xml(r)));
    }
    if let Some(s) = &source {
        optional_lines.push(format!("    <dc:source>{}</dc:source>", escape_xml(s)));
    }
    if let Some(r) = &relation {
        optional_lines.push(format!("    <dc:relation>{}</dc:relation>", escape_xml(r)));
    }
    if let Some(c) = &coverage {
        optional_lines.push(format!("    <dc:coverage>{}</dc:coverage>", escape_xml(c)));
    }
    if !optional_lines.is_empty() {
        other_blocks.push(optional_lines.join("\n"));
    }
    let other_metadata = other_blocks.join("\n\n");

    // contributor 块处理：若无则置空（后续会清理空的 contributor 标签）
    let contributor_val = contributor.unwrap_or_default();

    // 先替换主要占位（OTHER_METADATA 统一，兼容旧 BOOK_DESCRIPTION/BOOK_SUBJECTS）
    opf_tpl = opf_tpl
        .replace("{{BOOK_UUID}}", &escape_xml(&book_uuid))
        .replace("{{BOOK_TITLE}}", &escape_xml(&title))
        .replace("{{BOOK_MODIFIED}}", &escape_xml(&modified))
        .replace("{{BOOK_AUTHOR}}", &escape_xml(&book_author))
        .replace("{{CONTRIBUTOR}}", &escape_xml(&contributor_val))
        .replace("{{OTHER_METADATA}}", &other_metadata)
        .replace("{{BOOK_DESCRIPTION}}", "")
        .replace("{{BOOK_SUBJECTS}}", "")
        .replace("{{COVER_ITEM}}", &cover_item)
        .replace("{{MANIFEST_ITEMS}}", &manifest_items)
        .replace("{{SPINE_ITEMS}}", &spine_items)
        .replace("{{NAV_ITEMREF}}", "")
        .replace("{{BOOK_LANGUAGE}}", &escape_xml(&language_esc));

    // 兼容旧模板硬编码 zh-CN -> 替换
    if !opf_tpl.contains("{{BOOK_LANGUAGE}}") {
        // 模板未使用占位，直接替换硬编码
        opf_tpl = opf_tpl.replace(r#"xml:lang="zh-CN""#, &format!(r#"xml:lang="{}""#, escape_xml(&language_esc)));
        opf_tpl = opf_tpl.replace(
            "<dc:language>zh-CN</dc:language>",
            &format!("<dc:language>{}</dc:language>", escape_xml(&language_esc)),
        );
    }

    // 清理空的 contributor 段（若 contributor 为空，删除该块）
    if contributor_val.is_empty() {
        // 移除 <dc:contributor ...></dc:contributor> + 下一行 meta
        // 简单替换空标签为 ""
        opf_tpl = opf_tpl.replace(r##"    <dc:contributor id="contributor"></dc:contributor>"##, "");
        opf_tpl = opf_tpl.replace(
            r##"    <meta refines="#contributor" property="role" scheme="marc:relators">edt</meta>"##,
            "",
        );
        // 清理多余空行
        while opf_tpl.contains("\n\n\n") {
            opf_tpl = opf_tpl.replace("\n\n\n", "\n\n");
        }
    }

    fs::write(epub_dir.join("content.opf"), opf_tpl)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 content.opf 失败: {e}")))?;

    // nav.xhtml
    let mut nav_tpl = read_template(app, "EPUB33-NOVEL/EPUB/nav.xhtml")?;
    let mut toc_list = String::new();
    let mut landmarks = String::new();
    landmarks.push_str(r#"      <li><a epub:type="cover" href="cover.xhtml">封面</a></li>"#);
    landmarks.push('\n');
    landmarks.push_str(r#"      <li><a epub:type="titlepage" href="titlepage.xhtml">扉页</a></li>"#);
    if let Some(first) = chap_infos.first() {
        landmarks.push('\n');
        landmarks.push_str(&format!(
            r#"      <li><a epub:type="bodymatter" href="{}">正文</a></li>"#,
            first.href
        ));
    }
    // TOC: 若有卷则卷为一级，章为二级
    if !vol_infos.is_empty() {
        for vol in &vol_infos {
            toc_list.push_str(&format!(
                r#"      <li><a href="{}">{}</a>"#,
                vol.href,
                escape_xml(&vol.display_title)
            ));
            // 该卷下的章
            let chaps_for_vol: Vec<&ChapInfo> = chap_infos.iter().filter(|c| c.vol_order == Some(vol.order)).collect();
            if !chaps_for_vol.is_empty() {
                toc_list.push_str("\n        <ol>\n");
                for chap in chaps_for_vol {
                    toc_list.push_str(&format!(
                        r#"          <li><a href="{}">{}</a></li>"#,
                        chap.href,
                        escape_xml(&chap.display_title)
                    ));
                    toc_list.push('\n');
                }
                toc_list.push_str("        </ol>\n");
            }
            toc_list.push_str("      </li>\n");
        }
        for chap in chap_infos.iter().filter(|c| c.vol_order.is_none()) {
            // 卷外章（纯章节遗漏）直接一级
            toc_list.push_str(&format!(
                r#"      <li><a href="{}">{}</a></li>"#,
                chap.href,
                escape_xml(&chap.display_title)
            ));
            toc_list.push('\n');
        }
    } else {
        for chap in &chap_infos {
            toc_list.push_str(&format!(
                r#"      <li><a href="{}">{}</a></li>"#,
                chap.href,
                escape_xml(&chap.display_title)
            ));
            toc_list.push('\n');
        }
    }
    // 去尾换行
    if toc_list.ends_with('\n') {
        toc_list.pop();
    }
    if landmarks.ends_with('\n') {
        landmarks.pop();
    }
    nav_tpl = nav_tpl
        .replace("{{CHAPTER_TOC_LIST}}", &toc_list)
        .replace("{{LANDMARKS_LIST}}", &landmarks);
    fs::write(epub_dir.join("nav.xhtml"), nav_tpl)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 nav 失败: {e}")))?;

    // cover.xhtml
    let mut cover_tpl = read_template(app, "EPUB33-NOVEL/EPUB/cover.xhtml")?;
    cover_tpl = cover_tpl
        .replace("{{BOOK_TITLE}}", &escape_xml(&title))
        .replace("{{COVER_BLOCK}}", &cover_block);
    fs::write(epub_dir.join("cover.xhtml"), cover_tpl)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 cover 失败: {e}")))?;

    // titlepage.xhtml
    let mut title_tpl = read_template(app, "EPUB33-NOVEL/EPUB/titlepage.xhtml")?;
    let desc_block = if descriptions.is_empty() {
        "".to_string()
    } else {
        let inner = descriptions
            .iter()
            .map(|d| format!(r#"<span class="desc-line">{}</span>"#, escape_xml(d)))
            .collect::<Vec<_>>()
            .join("\n");
        format!(r#"<div class="book-description">{inner}</div>"#)
    };
    title_tpl = title_tpl
        .replace("{{BOOK_TITLE}}", &escape_xml(&title))
        .replace("{{BOOK_AUTHOR}}", &escape_xml(&book_author))
        .replace("{{BOOK_DESCRIPTION_BLOCK}}", &desc_block);
    fs::write(epub_dir.join("titlepage.xhtml"), title_tpl)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入 titlepage 失败: {e}")))?;

    // build 完成，生成文件树
    let files = collect_build_files(&book_root, &book_root)?;
    let result = BuildResult {
        epub_path: book_root.to_string_lossy().to_string(),
        files,
        book_title: title.clone(),
    };
    log::info!("[build] built {} -> {}", id, book_root.display());
    Ok(result)
}

/// 读取构建文件内容（仅文本）
pub(crate) fn read_build_file(app: &AppHandle, identifier: &str, rel_path: &str) -> AppResult<String> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let rel = rel_path.trim().trim_start_matches('/').trim_start_matches('\\').to_string();
    if rel.is_empty() || rel.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[build] 非法路径"));
    }
    let (project_dir, _uuid, _) = resolve_project_base(app, &id)?;
    // 找到 book_root
    let Some(root) = find_book_root(&project_dir)? else {
        return Err(AppError::new(CODE_ERROR, "[build] 尚未构建"));
    };
    let target = root.join(&rel);
    // 防止越界
    let canon_root = root.canonicalize().unwrap_or(root.clone());
    let canon_target = target.canonicalize().unwrap_or(target.clone());
    if !canon_target.starts_with(&canon_root) {
        return Err(AppError::new(CODE_ERROR, "[build] 非法路径"));
    }
    if !target.exists() || !target.is_file() {
        return Err(AppError::new(CODE_ERROR, "[build] 文件不存在"));
    }
    if !is_text_file(&target) {
        return Err(AppError::new(CODE_ERROR, "[build] 仅支持编辑文本文件"));
    }
    let bytes = fs::read(&target).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 读取失败: {e}")))?;
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(AppError::new(CODE_ERROR, "[build] 文件过大（>5MB）"));
    }
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        Ok(s)
    } else {
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// 写入构建文件内容（仅文本）
pub(crate) fn write_build_file(app: &AppHandle, identifier: &str, rel_path: &str, content: String) -> AppResult<bool> {
    let id = identifier.trim().to_string();
    if id.is_empty() {
        return Err(AppError::new(CODE_ERROR, "[build] identifier 不能为空"));
    }
    let rel = rel_path.trim().trim_start_matches('/').trim_start_matches('\\').to_string();
    if rel.is_empty() || rel.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[build] 非法路径"));
    }
    let (project_dir, _uuid, _) = resolve_project_base(app, &id)?;
    let Some(root) = find_book_root(&project_dir)? else {
        return Err(AppError::new(CODE_ERROR, "[build] 尚未构建"));
    };
    let target = root.join(&rel);
    let canon_root = root.canonicalize().unwrap_or(root.clone());
    // 对于不存在文件，取 parent canonical
    let parent = target.parent().unwrap_or(&root);
    let canon_parent = parent.canonicalize().unwrap_or(parent.to_path_buf());
    if !canon_parent.starts_with(&canon_root) && canon_parent != canon_root {
        return Err(AppError::new(CODE_ERROR, "[build] 非法路径"));
    }
    if !is_text_file(&target) {
        return Err(AppError::new(CODE_ERROR, "[build] 仅支持编辑文本文件"));
    }
    if let Some(p) = target.parent() {
        fs::create_dir_all(p).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 创建目录失败: {e}")))?;
    }
    fs::write(&target, content).map_err(|e| AppError::new(CODE_ERROR, format!("[build] 写入失败: {e}")))?;
    log::info!("[build] write {} for {}", rel, id);
    Ok(true)
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

#[cfg(test)]
mod tests {
    use super::{apply_title_format, compute_pad_width, escape_xml, format_number_display, sanitize_title};

    #[test]
    fn test_escape() {
        assert_eq!(escape_xml("a&b<\"c\""), "a&amp;b&lt;&quot;c&quot;");
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_title(" a/b:c*? "), "a_b_c__");
        assert_eq!(sanitize_title(""), "untitled");
        assert_eq!(sanitize_title("正常书名"), "正常书名");
    }

    #[test]
    fn test_pad_width() {
        assert_eq!(compute_pad_width(9), 2);
        assert_eq!(compute_pad_width(10), 2);
        assert_eq!(compute_pad_width(100), 3);
        assert_eq!(compute_pad_width(999), 3);
        assert_eq!(compute_pad_width(1000), 4);
        assert_eq!(compute_pad_width(0), 2);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number_display(5, "arabic", 3), "5");
        assert_eq!(format_number_display(5, "arabic_padded", 3), "005");
        assert_eq!(format_number_display(12, "arabic_padded", 2), "12");
        // 中文小写/大写不补零
        assert_eq!(format_number_display(1, "chinese_lower", 3), "一");
        assert_eq!(format_number_display(2, "chinese_upper", 3), "贰");
    }

    #[test]
    fn test_apply_format() {
        assert_eq!(apply_title_format("第{order}章 {title}", "5", "风起"), "第5章 风起");
        assert_eq!(apply_title_format("{title}-{order}", "001", "序"), "序-001");
        assert_eq!(apply_title_format("前言", "1", "前言"), "前言");
    }
}
