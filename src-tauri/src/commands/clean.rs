//! 清洗命令：IPC 薄层，转发至 `features::clean`。

use tauri::AppHandle;

use crate::cores::response::Response;
use crate::features::clean::{TxtDetail, TxtInfo};

/// 列出 txts 目录文件：`invokeCommand("list_txts")`
#[tauri::command]
pub fn list_txts(app: AppHandle) -> Response<Vec<TxtInfo>> {
    crate::features::clean::list_txts(&app).into()
}

/// 复制文件/目录到 txts：`invokeCommand("copy_txt", { srcPath })`
/// 支持单文件与文件夹递归，返回实际复制的文件列表（重命名后名称）
#[tauri::command]
pub fn copy_txt(app: AppHandle, src_path: String) -> Response<Vec<TxtInfo>> {
    crate::features::clean::copy_txt(&app, &src_path).into()
}

/// 获取详情：`invokeCommand("get_txt_detail", { name })`
#[tauri::command]
pub fn get_txt_detail(app: AppHandle, name: String) -> Response<TxtDetail> {
    crate::features::clean::get_txt_detail(&app, &name).into()
}

/// 删除：`invokeCommand("delete_txt", { name })`
#[tauri::command]
pub fn delete_txt(app: AppHandle, name: String) -> Response<bool> {
    crate::features::clean::delete_txt(&app, &name).into()
}
