// release 构建下隐藏 Windows 控制台窗口（仅 GUI 运行）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 实际逻辑位于 lib.rs 的 run()，便于移动端入口复用同一实现
    ebook_studio_lib::run()
}
