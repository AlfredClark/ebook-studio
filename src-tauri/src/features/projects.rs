//! 项目业务逻辑：基于文件系统的项目管理（Dublin Core 元数据）
//!
//! 真相源：`APPDATA/Projects/<uuid>/metadata.json + sources/{cover,content}`
//! - identifier 为 `urn:uuid:<uuid>`，目录名仅用 `<uuid>` 段
//! - cover 固定为 `sources/cover.<ext>`，content 固定为 `sources/content.txt`
//! - metadata.json 承载全部 Dublin 字段 + 系统时间 + 资产相对路径
//! - 列表通过扫描 Projects 目录读取 metadata.json 聚合
//!
//! 已知边界：
//! - identifier 去 `urn:uuid:` 前缀后即目录名，非法字符已由 Uuid 保证合法
//! - subjects 以 "/" 分割存数组，description 以换行分割存数组，空则 []
//! - 大文件拷贝未做进度，同步阻塞

#![allow(clippy::collapsible_if, clippy::unnecessary_sort_by)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::cores::response::{AppError, AppResult, CODE_ERROR};

const PROJECTS_DIR: &str = "Projects";
const METADATA_FILE: &str = "metadata.json";
const SOURCES_DIR: &str = "sources";
const COVER_PREFIX: &str = "cover";
const CONTENT_NAME: &str = "content.txt";

/// 项目元数据（与前端 Project 类型对齐，camelCase 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    /// 标识符 `urn:uuid:<uuid>`
    pub identifier: String,
    /// 标题（必填）
    pub title: String,
    /// 语言（必填，en | zh-CN）
    pub language: String,
    /// 创作者
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// 出版/创建日期（YYYY-MM-DD）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// 主题数组（"/" 分割后）
    #[serde(default)]
    pub subjects: Vec<String>,
    /// 简介数组（每行一个元素）
    #[serde(default)]
    pub description: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<String>,
    /// 系统创建时间（RFC3339）
    pub created: String,
    /// 系统修改时间（RFC3339）
    pub modified: String,
    /// 修改时间戳毫秒（便于前端排序兼容旧 modified: number）
    pub modified_ms: i64,
    /// 封面相对路径 `sources/cover.<ext>`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    /// 正文相对路径 `sources/content.txt`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// 创建项目输入（前端 camelCase）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub title: String,
    pub language: String,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub contributor: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    /// 前端原始 subjects 字符串（"/" 分割），后端拆数组
    #[serde(default)]
    pub subjects: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rights: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub relation: Option<String>,
    #[serde(default)]
    pub coverage: Option<String>,
    /// 封面原始绝对路径（前端 dialog 选取）
    #[serde(default)]
    pub cover_src: Option<String>,
    /// 正文原始绝对路径
    #[serde(default)]
    pub content_src: Option<String>,
}

/// 更新项目输入（前端 camelCase，需 identifier；封面/正文用 Option 区分保留/替换/移除）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub identifier: String,
    pub title: String,
    pub language: String,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub contributor: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub subjects: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rights: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub relation: Option<String>,
    #[serde(default)]
    pub coverage: Option<String>,
    /// 封面原始绝对路径（Some(path) 替换，None 保留；配合 remove_cover 删除）
    #[serde(default)]
    pub cover_src: Option<String>,
    #[serde(default)]
    pub content_src: Option<String>,
    /// 是否移除现有封面（cover_src 为空时生效）
    #[serde(default)]
    pub remove_cover: Option<bool>,
    /// 是否移除现有正文
    #[serde(default)]
    pub remove_content: Option<bool>,
}

fn projects_base_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .resolve(PROJECTS_DIR, BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[projects] resolve Projects dir 失败: {e}")))
}

fn ensure_projects_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = projects_base_dir(app)?;
    fs::create_dir_all(&dir).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 创建 Projects 目录失败: {e}")))?;
    Ok(dir)
}

fn validate_title(title: &str) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::new(CODE_ERROR, "[projects] 标题不能为空"));
    }
    if title.trim().len() > 500 {
        return Err(AppError::new(CODE_ERROR, "[projects] 标题过长"));
    }
    Ok(())
}

fn validate_language(language: &str) -> AppResult<()> {
    match language {
        "en" | "zh-CN" => Ok(()),
        _ => Err(AppError::new(CODE_ERROR, "[projects] 语言仅支持 en / zh-CN")),
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    match value {
        Some(v) => {
            let t = v.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        }
        None => None,
    }
}

fn parse_subjects(raw: Option<String>) -> Vec<String> {
    match raw {
        Some(s) => s.split('/').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
        None => Vec::new(),
    }
}

fn parse_description(raw: Option<String>) -> Vec<String> {
    match raw {
        Some(s) => s.lines().map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
        None => Vec::new(),
    }
}

fn sanitize_uuid(identifier: &str) -> AppResult<String> {
    // identifier 形如 urn:uuid:<uuid>，目录名取最后段
    let uuid_part = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:")
    } else {
        identifier
    };
    // 校验 uuid 合法
    Uuid::parse_str(uuid_part).map_err(|_| AppError::new(CODE_ERROR, "[projects] 非法 identifier"))?;
    // 防路径遍历
    if uuid_part.contains('/') || uuid_part.contains('\\') || uuid_part.contains("..") {
        return Err(AppError::new(CODE_ERROR, "[projects] 非法 identifier"));
    }
    Ok(uuid_part.to_string())
}

fn metadata_path(project_dir: &Path) -> PathBuf {
    project_dir.join(METADATA_FILE)
}

fn read_metadata_file(path: &Path) -> AppResult<ProjectMetadata> {
    let data =
        fs::read_to_string(path).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 读取 metadata.json 失败: {e}")))?;
    serde_json::from_str(&data).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 解析 metadata.json 失败: {e}")))
}

/// 列出全部项目（扫描 Projects 目录，损坏的跳过）
pub(crate) fn list_projects(app: &AppHandle) -> AppResult<Vec<ProjectMetadata>> {
    let base = ensure_projects_dir(app)?;
    let mut out = Vec::new();
    let entries =
        fs::read_dir(&base).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 读取 Projects 目录失败: {e}")))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta = metadata_path(&path);
        if !meta.exists() {
            continue;
        }
        match read_metadata_file(&meta) {
            Ok(m) => out.push(m),
            Err(e) => {
                log::warn!("[projects] 跳过损坏项目 {}: {e}", path.display());
                continue;
            }
        }
    }
    // 默认最新修改优先
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(out)
}

/// 创建项目
pub(crate) fn create_project(app: &AppHandle, input: CreateProjectInput) -> AppResult<ProjectMetadata> {
    let title = input.title.trim().to_string();
    validate_title(&title)?;
    let language = input.language.trim().to_string();
    validate_language(&language)?;

    let subjects = parse_subjects(input.subjects);
    let description = parse_description(input.description);

    // 生成 identifier 与时间
    let uuid = Uuid::new_v4().to_string();
    let identifier = format!("urn:uuid:{uuid}");
    let now = Utc::now();
    let now_rfc = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let now_ms = now.timestamp_millis();

    let base = ensure_projects_dir(app)?;
    let project_dir = base.join(&uuid);
    if project_dir.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 项目已存在"));
    }
    fs::create_dir_all(&project_dir).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 创建项目目录失败: {e}")))?;
    let sources_dir = project_dir.join(SOURCES_DIR);
    fs::create_dir_all(&sources_dir)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 创建 sources 目录失败: {e}")))?;

    // 处理封面与正文拷贝（失败回滚目录）
    let (cover_rel, content_rel) = {
        let do_create = || -> AppResult<(Option<String>, Option<String>)> {
            // 封面
            let mut c_rel = None;
            if let Some(src) = normalize_optional(input.cover_src) {
                let src_path = Path::new(&src);
                if !src_path.exists() {
                    return Err(AppError::new(CODE_ERROR, "[projects] 封面文件不存在"));
                }
                if !src_path.is_file() {
                    return Err(AppError::new(CODE_ERROR, "[projects] 封面不是文件"));
                }
                let ext = src_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_else(|| "jpg".to_string());
                // 仅允许图片扩展
                let allowed = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];
                if !allowed.contains(&ext.as_str()) {
                    return Err(AppError::new(CODE_ERROR, "[projects] 封面仅支持图片格式"));
                }
                let dest_name = format!("{COVER_PREFIX}.{ext}");
                let dest = sources_dir.join(&dest_name);
                fs::copy(src_path, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 复制封面失败: {e}")))?;
                c_rel = Some(format!("{SOURCES_DIR}/{dest_name}"));
            }

            // 正文 txt
            let mut t_rel = None;
            if let Some(src) = normalize_optional(input.content_src) {
                let src_path = Path::new(&src);
                if !src_path.exists() {
                    return Err(AppError::new(CODE_ERROR, "[projects] 正文文件不存在"));
                }
                if !src_path.is_file() {
                    return Err(AppError::new(CODE_ERROR, "[projects] 正文不是文件"));
                }
                // 校验扩展名为 txt（可放宽）
                if let Some(ext) = src_path.extension().and_then(|e| e.to_str()) {
                    if !ext.eq_ignore_ascii_case("txt") {
                        return Err(AppError::new(CODE_ERROR, "[projects] 正文仅支持 txt 格式"));
                    }
                }
                let dest = sources_dir.join(CONTENT_NAME);
                fs::copy(src_path, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 复制正文失败: {e}")))?;
                t_rel = Some(format!("{SOURCES_DIR}/{CONTENT_NAME}"));
            }
            Ok((c_rel, t_rel))
        };
        match do_create() {
            Ok(v) => v,
            Err(e) => {
                // 回滚目录
                let _ = fs::remove_dir_all(&project_dir);
                return Err(e);
            }
        }
    };

    let metadata = ProjectMetadata {
        identifier: identifier.clone(),
        title,
        language,
        creator: normalize_optional(input.creator),
        contributor: normalize_optional(input.contributor),
        publisher: normalize_optional(input.publisher),
        date: normalize_optional(input.date),
        subjects,
        description,
        rights: normalize_optional(input.rights),
        source: normalize_optional(input.source),
        relation: normalize_optional(input.relation),
        coverage: normalize_optional(input.coverage),
        created: now_rfc.clone(),
        modified: now_rfc,
        modified_ms: now_ms,
        cover: cover_rel,
        content: content_rel,
    };

    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 序列化失败: {e}")))?;
    let meta_path = metadata_path(&project_dir);
    if let Err(e) = fs::write(&meta_path, json) {
        let _ = fs::remove_dir_all(&project_dir);
        return Err(AppError::new(CODE_ERROR, format!("[projects] 写入 metadata.json 失败: {e}")));
    }

    log::info!("[projects] created {}", identifier);
    Ok(metadata)
}

fn remove_existing_covers(sources_dir: &Path) {
    if let Ok(entries) = fs::read_dir(sources_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(COVER_PREFIX) && p.is_file() {
                    let _ = fs::remove_file(&p);
                }
            }
        }
    }
}

/// 更新项目（保留 created，刷新 modified；封面/正文支持替换/保留/移除）
pub(crate) fn update_project(app: &AppHandle, input: UpdateProjectInput) -> AppResult<ProjectMetadata> {
    let title = input.title.trim().to_string();
    validate_title(&title)?;
    let language = input.language.trim().to_string();
    validate_language(&language)?;
    let subjects = parse_subjects(input.subjects);
    let description = parse_description(input.description);
    let uuid = sanitize_uuid(&input.identifier)?;
    let base = ensure_projects_dir(app)?;
    let project_dir = base.join(&uuid);
    if !project_dir.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 项目不存在"));
    }
    let meta_path = metadata_path(&project_dir);
    let existing = read_metadata_file(&meta_path)?;
    let sources_dir = project_dir.join(SOURCES_DIR);
    fs::create_dir_all(&sources_dir)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 创建 sources 目录失败: {e}")))?;

    // 封面：cover_src 优先级 > remove_cover > 保留
    let cover_rel: Option<String> = if let Some(src) = normalize_optional(input.cover_src) {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            return Err(AppError::new(CODE_ERROR, "[projects] 封面文件不存在"));
        }
        if !src_path.is_file() {
            return Err(AppError::new(CODE_ERROR, "[projects] 封面不是文件"));
        }
        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_else(|| "jpg".to_string());
        let allowed = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];
        if !allowed.contains(&ext.as_str()) {
            return Err(AppError::new(CODE_ERROR, "[projects] 封面仅支持图片格式"));
        }
        // 先清理旧封面（不同扩展残留）
        remove_existing_covers(&sources_dir);
        let dest_name = format!("{COVER_PREFIX}.{ext}");
        let dest = sources_dir.join(&dest_name);
        fs::copy(src_path, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 复制封面失败: {e}")))?;
        Some(format!("{SOURCES_DIR}/{dest_name}"))
    } else if input.remove_cover.unwrap_or(false) {
        remove_existing_covers(&sources_dir);
        None
    } else {
        existing.cover.clone()
    };

    // 正文：content_src 优先级 > remove_content > 保留
    let content_rel: Option<String> = if let Some(src) = normalize_optional(input.content_src) {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            return Err(AppError::new(CODE_ERROR, "[projects] 正文文件不存在"));
        }
        if !src_path.is_file() {
            return Err(AppError::new(CODE_ERROR, "[projects] 正文不是文件"));
        }
        if let Some(ext) = src_path.extension().and_then(|e| e.to_str()) {
            if !ext.eq_ignore_ascii_case("txt") {
                return Err(AppError::new(CODE_ERROR, "[projects] 正文仅支持 txt 格式"));
            }
        }
        let dest = sources_dir.join(CONTENT_NAME);
        fs::copy(src_path, &dest).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 复制正文失败: {e}")))?;
        Some(format!("{SOURCES_DIR}/{CONTENT_NAME}"))
    } else if input.remove_content.unwrap_or(false) {
        let dest = sources_dir.join(CONTENT_NAME);
        if dest.exists() {
            let _ = fs::remove_file(&dest);
        }
        None
    } else {
        existing.content.clone()
    };

    let now = Utc::now();
    let now_rfc = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let now_ms = now.timestamp_millis();

    let metadata = ProjectMetadata {
        identifier: existing.identifier.clone(),
        title,
        language,
        creator: normalize_optional(input.creator),
        contributor: normalize_optional(input.contributor),
        publisher: normalize_optional(input.publisher),
        date: normalize_optional(input.date),
        subjects,
        description,
        rights: normalize_optional(input.rights),
        source: normalize_optional(input.source),
        relation: normalize_optional(input.relation),
        coverage: normalize_optional(input.coverage),
        created: existing.created.clone(),
        modified: now_rfc,
        modified_ms: now_ms,
        cover: cover_rel,
        content: content_rel,
    };

    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 序列化失败: {e}")))?;
    fs::write(&meta_path, json).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 写入 metadata.json 失败: {e}")))?;
    log::info!("[projects] updated {}", existing.identifier);
    Ok(metadata)
}

/// 删除单个项目（整目录）
pub(crate) fn delete_project(app: &AppHandle, identifier: &str) -> AppResult<()> {
    let uuid = sanitize_uuid(identifier)?;
    let base = ensure_projects_dir(app)?;
    let dir = base.join(&uuid);
    if !dir.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 项目不存在"));
    }
    fs::remove_dir_all(&dir).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 删除失败: {e}")))?;
    log::info!("[projects] deleted {}", identifier);
    Ok(())
}

/// 批量删除
pub(crate) fn batch_delete_projects(app: &AppHandle, identifiers: Vec<String>) -> AppResult<usize> {
    if identifiers.is_empty() {
        return Ok(0);
    }
    let base = ensure_projects_dir(app)?;
    let mut ok = 0usize;
    let mut last_err: Option<AppError> = None;
    for id in identifiers {
        match sanitize_uuid(&id) {
            Ok(uuid) => {
                let dir = base.join(&uuid);
                if dir.exists() {
                    match fs::remove_dir_all(&dir) {
                        Ok(_) => {
                            ok += 1;
                            log::info!("[projects] batch deleted {}", id);
                        }
                        Err(e) => {
                            log::warn!("[projects] batch delete 失败 {}: {e}", id);
                            last_err = Some(AppError::new(CODE_ERROR, format!("[projects] 删除 {id} 失败: {e}")));
                        }
                    }
                } else {
                    log::warn!("[projects] batch delete 跳过不存在 {}", id);
                }
            }
            Err(e) => {
                log::warn!("[projects] 非法 identifier 跳过 {}: {e}", id);
                last_err = Some(e);
            }
        }
    }
    if ok == 0 {
        if let Some(e) = last_err {
            return Err(e);
        }
    }
    Ok(ok)
}

/// 获取单个项目详情（按 identifier）
pub(crate) fn get_project(app: &AppHandle, identifier: &str) -> AppResult<ProjectMetadata> {
    let uuid = sanitize_uuid(identifier)?;
    let base = ensure_projects_dir(app)?;
    let dir = base.join(&uuid);
    let meta = metadata_path(&dir);
    if !meta.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 项目不存在"));
    }
    read_metadata_file(&meta)
}

/// 将 Projects/<uuid>/sources/... 相对路径解析为绝对路径（供前端 convertFileSrc）
pub(crate) fn resolve_asset_path(app: &AppHandle, identifier: &str, relative: &str) -> AppResult<String> {
    let uuid = sanitize_uuid(identifier)?;
    let base = ensure_projects_dir(app)?;
    let abs = base.join(&uuid).join(relative);
    // 防遍历：确保仍在 base 内
    let canonical_base = base.canonicalize().unwrap_or(base.clone());
    let canonical_abs = abs.canonicalize().unwrap_or(abs.clone());
    if !canonical_abs.starts_with(&canonical_base) {
        return Err(AppError::new(CODE_ERROR, "[projects] 非法资产路径"));
    }
    Ok(canonical_abs.to_string_lossy().to_string())
}

/// 文件统计（用于正文预览：大小与字符数）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStats {
    pub size: u64,
    pub chars: usize,
}

/// 获取任意路径文件的统计（大小与字符数），用于创建页正文预览
pub(crate) fn get_file_stats(path: &str) -> AppResult<FileStats> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 文件不存在"));
    }
    if !p.is_file() {
        return Err(AppError::new(CODE_ERROR, "[projects] 不是文件"));
    }
    let metadata = fs::metadata(p).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 读取文件信息失败: {e}")))?;
    let size = metadata.len();
    // 字符数：尝试按 utf8 读取，失败则按字节长度
    let chars = match fs::read_to_string(p) {
        Ok(s) => s.chars().count(),
        Err(_) => match fs::read(p) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).chars().count(),
            Err(_) => 0,
        },
    };
    Ok(FileStats { size, chars })
}

/// 读取图片文件为 data URL（用于创建页封面预览，规避 asset 协议限制）
pub(crate) fn read_image_as_data_url(path: &str) -> AppResult<String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(CODE_ERROR, "[projects] 封面文件不存在"));
    }
    if !p.is_file() {
        return Err(AppError::new(CODE_ERROR, "[projects] 封面不是文件"));
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "png".to_string());
    let allowed = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];
    if !allowed.contains(&ext.as_str()) {
        return Err(AppError::new(CODE_ERROR, "[projects] 封面仅支持图片格式"));
    }
    let bytes = fs::read(p).map_err(|e| AppError::new(CODE_ERROR, format!("[projects] 读取封面失败: {e}")))?;
    // 限制 10MB 以内预览，避免大图内存暴涨
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(AppError::new(CODE_ERROR, "[projects] 封面文件过大（>10MB）"));
    }
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
