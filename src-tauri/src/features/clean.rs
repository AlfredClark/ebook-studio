//! 清洗业务：APPDATA/txts 目录管理（复制/列表/详情/删除）。
//!
//! 真相源：`APPDATA/txts/`（`app.path().resolve("txts", BaseDirectory::AppData)` 解析，
//! 与 `config.json` 同属应用数据目录）。前端经 `dialog.open` 或 `onDragDropEvent` 获得
//! 源路径，交本模块完成落盘与去重。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::cores::response::{AppError, AppResult, CODE_ERROR};

/// 列表项：与前端 `TxtInfo` 对齐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxtInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub mtime: u64,
}

/// 详情：含行数与字符统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxtDetail {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub mtime: u64,
    pub lines: usize,
    pub chars: usize,
}

/// 解析 txts 目录路径
fn txts_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .resolve("txts", BaseDirectory::AppData)
        .map_err(|e| AppError::new(CODE_ERROR, format!("[clean] resolve txts dir 失败: {e}")))
}

/// 确保目录存在
pub(crate) fn ensure_txts_dir(app: &AppHandle) -> AppResult<()> {
    let dir = txts_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 创建目录失败: {e}")))?;
    Ok(())
}

/// 校验文件名合法（防路径穿越，仅允许单文件名）
fn validate_name(name: &str) -> AppResult<()> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        return Err(AppError::new(CODE_ERROR, "[clean] 非法文件名"));
    }
    Ok(())
}

/// 生成去重目标路径：name(1).txt 形态
fn unique_dest(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut n: u32 = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{stem}({n})")
        } else {
            format!("{stem}({n}).{ext}")
        };
        let p = dir.join(&new_name);
        if !p.exists() {
            return p;
        }
        n += 1;
        // 防止极端情况无限循环
        if n > 9999 {
            return dir.join(format!("{stem}({n}).{ext}"));
        }
    }
}

/// 将 SystemTime 转 unix 秒
fn mtime_to_secs(mtime: SystemTime) -> u64 {
    mtime.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 构造 TxtInfo
fn to_info(path: &Path) -> AppResult<TxtInfo> {
    let metadata = std::fs::metadata(path).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 读取元信息失败: {e}")))?;
    let size = metadata.len();
    let mtime = metadata.modified().map(mtime_to_secs).unwrap_or(0);
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
    Ok(TxtInfo {
        name,
        path: path.to_string_lossy().to_string(),
        size,
        mtime,
    })
}

/// 递归收集 txt 文件
fn collect_txt_files(src: &Path, out: &mut Vec<PathBuf>) {
    if src.is_file() {
        if src
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
        {
            out.push(src.to_path_buf());
        }
        return;
    }
    if src.is_dir() {
        let Ok(entries) = std::fs::read_dir(src) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_txt_files(&p, out);
            } else if p.is_file()
                && p.extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
            {
                out.push(p);
            }
        }
    }
}

/// 列出 txts 目录下所有 txt 文件
#[allow(clippy::collapsible_if)]
pub(crate) fn list_txts(app: &AppHandle) -> AppResult<Vec<TxtInfo>> {
    ensure_txts_dir(app)?;
    let dir = txts_dir(app)?;
    let mut infos = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 读取目录失败: {e}")))?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file()
            && p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
        {
            if let Ok(info) = to_info(&p) {
                infos.push(info);
            }
        }
    }
    // 默认按 mtime 倒序（新的在前），与前端默认排序一致
    infos.sort_by_key(|info| std::cmp::Reverse(info.mtime));
    Ok(infos)
}

/// 复制文件或目录（递归）到 txts
pub(crate) fn copy_txt(app: &AppHandle, src_path: &str) -> AppResult<Vec<TxtInfo>> {
    ensure_txts_dir(app)?;
    let src = Path::new(src_path);
    if !src.exists() {
        return Err(AppError::new(CODE_ERROR, format!("[clean] 源路径不存在: {src_path}")));
    }
    let mut src_files = Vec::new();
    collect_txt_files(src, &mut src_files);
    if src_files.is_empty() {
        log::warn!("[clean] 未找到 txt 文件: {src_path}");
        return Ok(Vec::new());
    }
    let dir = txts_dir(app)?;
    let mut results = Vec::new();
    for src_file in src_files {
        let file_name = src_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file.txt")
            .to_string();
        let dest = unique_dest(&dir, &file_name);
        std::fs::copy(&src_file, &dest).map_err(|e| {
            AppError::new(
                CODE_ERROR,
                format!("[clean] 复制失败 {} -> {}: {e}", src_file.display(), dest.display()),
            )
        })?;
        log::info!("[clean] 已复制 {} -> {}", src_file.display(), dest.display());
        if let Ok(info) = to_info(&dest) {
            results.push(info);
        }
    }
    Ok(results)
}

/// 获取单个文件详情（含行数/字符数）
pub(crate) fn get_txt_detail(app: &AppHandle, name: &str) -> AppResult<TxtDetail> {
    validate_name(name)?;
    let dir = txts_dir(app)?;
    let path = dir.join(name);
    if !path.exists() || !path.is_file() {
        return Err(AppError::new(CODE_ERROR, format!("[clean] 文件不存在: {name}")));
    }
    let info = to_info(&path)?;
    // 统计行数与字符数（UTF-8 容错，失败则 0）
    let (lines, chars) = match std::fs::read_to_string(&path) {
        Ok(content) => (content.lines().count(), content.chars().count()),
        Err(e) => {
            log::warn!("[clean] 读取文件内容失败 {}: {e}", path.display());
            // 尝试按字节读取后 lossy 统计
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let lossy = String::from_utf8_lossy(&bytes);
                    (lossy.lines().count(), lossy.chars().count())
                }
                Err(_) => (0, 0),
            }
        }
    };
    Ok(TxtDetail {
        name: info.name,
        path: info.path,
        size: info.size,
        mtime: info.mtime,
        lines,
        chars,
    })
}

/// 删除文件
pub(crate) fn delete_txt(app: &AppHandle, name: &str) -> AppResult<bool> {
    validate_name(name)?;
    let dir = txts_dir(app)?;
    let path = dir.join(name);
    if !path.exists() {
        return Err(AppError::new(CODE_ERROR, format!("[clean] 文件不存在: {name}")));
    }
    std::fs::remove_file(&path).map_err(|e| AppError::new(CODE_ERROR, format!("[clean] 删除失败: {e}")))?;
    log::info!("[clean] 已删除 {}", path.display());
    Ok(true)
}
