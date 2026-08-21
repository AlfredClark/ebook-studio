//! 构建命令薄层

use tauri::AppHandle;

use crate::{
    cores::response::Response,
    features::build::{BuildResult, FormatConfig},
};

use crate::features::build as build_feat;

/// 获取已构建的 EPUB 目录（若不存在则 data 为 None）
/// 前端调用：`invokeCommand<BuildResult|null>("get_build", { identifier })`
#[tauri::command]
pub fn get_build(app: AppHandle, identifier: String) -> Response<Option<BuildResult>> {
    build_feat::get_build(&app, &identifier).into()
}

/// 执行构建（基于 split.json + metadata.json 生成未压缩 EPUB，支持标题/编号格式）
/// 前端调用：`invokeCommand<BuildResult>("build_epub", { identifier, chapterTitleFormat, volumeTitleFormat, numberFormat })`
#[tauri::command]
pub fn build_epub(
    app: AppHandle,
    identifier: String,
    chapter_title_format: Option<String>,
    volume_title_format: Option<String>,
    number_format: Option<String>,
) -> Response<BuildResult> {
    build_feat::build_epub(&app, &identifier, chapter_title_format, volume_title_format, number_format).into()
}

/// 删除构建目录（重新构建前）
/// 前端调用：`invokeCommand<boolean>("remove_build", { identifier })`
#[tauri::command]
pub fn remove_build(app: AppHandle, identifier: String) -> Response<bool> {
    match build_feat::remove_build(&app, &identifier) {
        Ok(()) => Response::ok(true),
        Err(e) => Response::err(e.code, e.message),
    }
}

/// 读取构建目录下指定文件的文本内容
/// 前端调用：`invokeCommand<string>("read_build_file", { identifier, relPath })`
#[tauri::command]
pub fn read_build_file(app: AppHandle, identifier: String, rel_path: String) -> Response<String> {
    build_feat::read_build_file(&app, &identifier, &rel_path).into()
}

/// 写入构建目录下指定文件的文本内容（仅文本文件）
/// 前端调用：`invokeCommand<boolean>("write_build_file", { identifier, relPath, content })`
#[tauri::command]
pub fn write_build_file(app: AppHandle, identifier: String, rel_path: String, content: String) -> Response<bool> {
    build_feat::write_build_file(&app, &identifier, &rel_path, content).into()
}

/// 获取构建目录绝对路径（供 opener 揭示）
/// 前端调用：`invokeCommand<string>("get_build_path", { identifier })`
#[tauri::command]
pub fn get_build_path(app: AppHandle, identifier: String) -> Response<Option<String>> {
    let r = build_feat::get_build(&app, &identifier).map(|opt| opt.map(|b| b.epub_path));
    r.into()
}

/// 获取已保存的格式化配置（format.json）
/// 前端调用：`invokeCommand<FormatConfig|null>("get_format", { identifier })`
#[tauri::command]
pub fn get_format(app: AppHandle, identifier: String) -> Response<Option<FormatConfig>> {
    build_feat::get_format(&app, &identifier).into()
}

/// 整目录一键格式化（xhtml/opf/xml/css，2 空格缩进，失败不落盘）
/// 前端调用：`invokeCommand<FormatBuildResult>("format_build_all", { identifier })`
#[tauri::command]
pub async fn format_build_all(app: AppHandle, identifier: String) -> Response<crate::features::build::FormatBuildResult> {
    let app_clone = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || build_feat::format_build_all(&app_clone, &identifier)).await;
    match res {
        Ok(inner) => inner.into(),
        Err(e) => Response::err(crate::cores::response::CODE_ERROR, format!("[build] 格式化任务失败: {e}")),
    }
}
