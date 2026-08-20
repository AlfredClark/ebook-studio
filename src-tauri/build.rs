// Tauri 构建脚本：生成窗口 context、图标与配置文件等产物，
// 由 Cargo 在编译前自动执行（见 src-tauri/Cargo.toml 的 build-dependencies）
fn main() {
    tauri_build::build()
}
