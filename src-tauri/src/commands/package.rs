//! 打包命令薄层

use tauri::AppHandle;

use crate::features::package as package_feat;
use crate::{cores::response::Response, features::package::PackageResult};

/// 查询已打包的产物（若 outputs 无 epub 则 None）
/// 前端调用：`invokeCommand<PackageResult|null>("get_package", { identifier })`
#[tauri::command]
pub fn get_package(app: AppHandle, identifier: String) -> Response<Option<PackageResult>> {
    package_feat::get_package(&app, &identifier).into()
}

/// 执行打包（压缩 build 目录为 epub + 生成 txt + 拷贝封面 + 校验）
/// 前端调用：`invokeCommand<PackageResult>("package_epub", { identifier })`
#[tauri::command]
pub async fn package_epub(app: AppHandle, identifier: String) -> Response<PackageResult> {
    let app_clone = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || package_feat::package_epub(&app_clone, &identifier)).await;
    match res {
        Ok(inner) => inner.into(),
        Err(e) => Response::err(crate::cores::response::CODE_ERROR, format!("[package] 任务失败: {e}")),
    }
}

/// 删除 outputs 目录（清空重建前或重置）
/// 前端调用：`invokeCommand<boolean>("remove_package", { identifier })`
#[tauri::command]
pub fn remove_package(app: AppHandle, identifier: String) -> Response<bool> {
    match package_feat::remove_package(&app, &identifier) {
        Ok(()) => Response::ok(true),
        Err(e) => Response::err(e.code, e.message),
    }
}

/// 校验已打包的 EPUB（按需触发，避免加载卡顿）
/// 前端调用：`invokeCommand<PackageResult>("verify_package", { identifier })`
#[tauri::command]
pub async fn verify_package(app: AppHandle, identifier: String) -> Response<PackageResult> {
    let app_clone = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || package_feat::verify_package(&app_clone, &identifier)).await;
    match res {
        Ok(inner) => inner.into(),
        Err(e) => Response::err(crate::cores::response::CODE_ERROR, format!("[package] 校验任务失败: {e}")),
    }
}

/// 获取 outputs 目录绝对路径（供 opener 揭示）
/// 前端调用：`invokeCommand<string|null>("get_package_path", { identifier })`
#[tauri::command]
pub fn get_package_path(app: AppHandle, identifier: String) -> Response<Option<String>> {
    package_feat::get_package_path(&app, &identifier).into()
}
