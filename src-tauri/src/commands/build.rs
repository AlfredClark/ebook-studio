//! 构建命令薄层

use tauri::AppHandle;

use crate::{cores::response::Response, features::build::BuildResult};

use crate::features::build as build_feat;

/// 获取已构建的 EPUB 目录（若不存在则 data 为 None）
/// 前端调用：`invokeCommand<BuildResult|null>("get_build", { identifier })`
#[tauri::command]
pub fn get_build(app: AppHandle, identifier: String) -> Response<Option<BuildResult>> {
    build_feat::get_build(&app, &identifier).into()
}

/// 执行构建（基于 split.json + metadata.json 生成未压缩 EPUB）
/// 前端调用：`invokeCommand<BuildResult>("build_epub", { identifier })`
#[tauri::command]
pub fn build_epub(app: AppHandle, identifier: String) -> Response<BuildResult> {
    build_feat::build_epub(&app, &identifier).into()
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
