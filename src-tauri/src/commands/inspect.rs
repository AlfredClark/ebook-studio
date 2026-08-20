//! 检查命令薄层

use tauri::AppHandle;

use crate::{
    cores::response::Response,
    features::inspect::{InspectRequest, InspectResult},
};

use crate::features::inspect as inspect_feat;

/// 检查 content 分卷分章
/// 前端调用：`invokeCommand<InspectResult>("inspect_content", { identifier, structure, volumeRegex, chapterRegex })`
#[tauri::command]
pub fn inspect_content(
    app: AppHandle,
    identifier: String,
    structure: Option<String>,
    volume_regex: Option<String>,
    chapter_regex: Option<String>,
) -> Response<InspectResult> {
    let req = InspectRequest {
        identifier,
        structure,
        volume_regex,
        chapter_regex,
    };
    inspect_feat::inspect_content(&app, req).into()
}

/// 重整章节/卷编号（仅改编号，保留标题）
/// 前端调用：`invokeCommand<InspectResult>("reorder_chapters", { identifier, structure, volumeRegex, chapterRegex })`
#[tauri::command]
pub fn reorder_chapters(
    app: AppHandle,
    identifier: String,
    structure: Option<String>,
    volume_regex: Option<String>,
    chapter_regex: Option<String>,
) -> Response<InspectResult> {
    let req = InspectRequest {
        identifier,
        structure,
        volume_regex,
        chapter_regex,
    };
    inspect_feat::reorder_chapters(&app, req).into()
}

/// 获取 content 绝对路径（供 opener 打开文件/文件夹）
/// 前端调用：`invokeCommand<string>("get_content_path", { identifier })`
#[tauri::command]
pub fn get_content_path(app: AppHandle, identifier: String) -> Response<String> {
    // 复用 inspect 的路径解析
    use crate::cores::response::{AppError, CODE_ERROR};
    use tauri::{Manager, path::BaseDirectory};
    let p = if identifier.starts_with("urn:uuid:") {
        identifier.trim_start_matches("urn:uuid:").to_string()
    } else {
        identifier.clone()
    };
    let uuid: String = match uuid::Uuid::parse_str(&p) {
        Ok(_) => p,
        Err(_) => {
            let e = AppError::new(CODE_ERROR, "[inspect] 非法 identifier");
            return Response::err(e.code, e.message);
        }
    };
    let base = match app.path().resolve("Projects", BaseDirectory::AppData) {
        Ok(p) => p,
        Err(e) => return Response::err(CODE_ERROR, format!("[inspect] resolve 失败: {e}")),
    };
    let path = base.join(&uuid).join("sources").join("content.txt");
    Response::ok(path.to_string_lossy().to_string())
}
