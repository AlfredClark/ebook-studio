//! 清理命令薄层

use tauri::AppHandle;

use crate::{
    cores::response::Response,
    features::clean::{CleanFilters, CleanResult},
};

use crate::features::clean as clean_feat;

/// 清理筛查（OR 语义，仅点击触发）
/// 前端调用：`invokeCommand<CleanResult>("filter_clean", { identifier, filters, customRegex })`
#[tauri::command]
pub fn filter_clean(
    app: AppHandle,
    identifier: String,
    filters: CleanFilters,
    custom_regex: Option<String>,
) -> Response<CleanResult> {
    clean_feat::filter_clean(&app, &identifier, filters, custom_regex).into()
}
