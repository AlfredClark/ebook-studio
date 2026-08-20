//! 项目命令薄层：参数校验 → 调 features → 转 Response

use tauri::AppHandle;

use crate::{
    cores::response::Response,
    features::projects::{self, CreateProjectInput, ProjectMetadata},
};

/// 列出全部项目
/// 前端调用：`invokeCommand<Project[]>("list_projects")`
#[tauri::command]
pub fn list_projects(app: AppHandle) -> Response<Vec<ProjectMetadata>> {
    projects::list_projects(&app).into()
}

/// 创建项目
/// 前端调用：`invokeCommand<Project>("create_project", { title, language, creator, ... })`
#[tauri::command]
pub fn create_project(app: AppHandle, payload: CreateProjectInput) -> Response<ProjectMetadata> {
    projects::create_project(&app, payload).into()
}

/// 删除单个项目（整目录）
/// 前端调用：`invokeCommand<boolean>("delete_project", { identifier })`
#[tauri::command]
pub fn delete_project(app: AppHandle, identifier: String) -> Response<bool> {
    match projects::delete_project(&app, &identifier) {
        Ok(()) => Response::ok(true),
        Err(e) => Response::err(e.code, e.message),
    }
}

/// 批量删除项目
/// 前端调用：`invokeCommand<number>("batch_delete_projects", { identifiers })`
#[tauri::command]
pub fn batch_delete_projects(app: AppHandle, identifiers: Vec<String>) -> Response<usize> {
    projects::batch_delete_projects(&app, identifiers).into()
}

/// 获取单个项目详情
/// 前端调用：`invokeCommand<Project>("get_project", { identifier })`
#[tauri::command]
pub fn get_project(app: AppHandle, identifier: String) -> Response<ProjectMetadata> {
    projects::get_project(&app, &identifier).into()
}

/// 解析项目资产为绝对路径（用于 convertFileSrc）
/// 前端调用：`invokeCommand<string>("resolve_project_asset", { identifier, relative })`
#[tauri::command]
pub fn resolve_project_asset(app: AppHandle, identifier: String, relative: String) -> Response<String> {
    projects::resolve_asset_path(&app, &identifier, &relative).into()
}

/// 获取文件统计（大小与字符数）
/// 前端调用：`invokeCommand<{size:number,chars:number}>("get_file_stats", { path })`
#[tauri::command]
pub fn get_file_stats(path: String) -> Response<projects::FileStats> {
    projects::get_file_stats(&path).into()
}

/// 读取图片为 data URL（用于封面预览）
/// 前端调用：`invokeCommand<string>("read_image_as_data_url", { path })`
#[tauri::command]
pub fn read_image_as_data_url(path: String) -> Response<String> {
    projects::read_image_as_data_url(&path).into()
}
