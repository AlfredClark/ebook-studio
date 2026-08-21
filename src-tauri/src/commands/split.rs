//! 拆分命令薄层

use tauri::AppHandle;

use crate::features::split::SplitResult;
use crate::{cores::response::Response, features::split as split_feat};

/// 获取已存在的拆分结果（若 split.json 不存在则 data 为 None）
/// 前端调用：`invokeCommand<SplitResult|null>("get_split_content", { identifier })`
#[tauri::command]
pub fn get_split_content(app: AppHandle, identifier: String) -> Response<Option<SplitResult>> {
    split_feat::get_split_content(&app, &identifier).into()
}

/// 执行拆分（解析 content.txt → 写 split.json）
/// 前端调用：`invokeCommand<SplitResult>("split_content", { identifier })`
#[tauri::command]
pub fn split_content(app: AppHandle, identifier: String) -> Response<SplitResult> {
    split_feat::split_content(&app, &identifier).into()
}

/// 保存章节内容（落盘 split.json）
/// 前端调用：`invokeCommand<SplitResult>("save_split_chapter", { identifier, volumeOrder, chapterOrder, contents })`
#[tauri::command]
pub fn save_split_chapter(
    app: AppHandle,
    identifier: String,
    volume_order: Option<i32>,
    chapter_order: i32,
    contents: Vec<String>,
) -> Response<SplitResult> {
    split_feat::save_split_chapter(&app, &identifier, volume_order, chapter_order, contents).into()
}
