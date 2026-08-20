//! 全局 panic hook：panic 信息写入共享日志链路（LogDir/stdout/Webview）。
//!
//! 已知边界：hook 安装于 Builder 创建前，logger 插件初始化前的早期 panic
//! 无法进入日志链路（log 宏静默丢弃），但默认 hook 的 stderr 输出仍保留。

use std::backtrace::Backtrace;
use std::panic;

/// 安装全局 panic hook：链式保留默认 hook（终端 stderr 输出），并额外
/// 以 `log::error!` 写入共享日志链路（LogDir 落盘 + 终端 + Webview devtools），
/// 附带 `Backtrace::capture()` 调用栈。进程行为保持 Rust panic 默认语义
/// （主线程 panic 终止进程、其他线程 panic 线程消亡），不做恢复或重启。
pub fn init_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        default_hook(info);
        log::error!("[panic] {info}");
        // 显式捕获调用栈（不受 RUST_BACKTRACE 环境变量影响）；release 无调试符号时帧数有限
        let backtrace = Backtrace::capture();
        log::error!("[panic] backtrace:\n{backtrace}");
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// 测试专用 logger：收集日志输出（测试进程内独立于 tauri-plugin-log 初始化）。
    struct Collector(Arc<Mutex<Vec<String>>>);

    impl log::Log for Collector {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            self.0.lock().unwrap().push(format!("[{}] {}", record.level(), record.args()));
        }

        fn flush(&self) {}
    }

    /// hook 将 panic 信息与 backtrace 写入日志链路。
    /// 注意：log::set_boxed_logger 为进程级全局操作，本测试须独占进程（当前仅此一个测试）。
    #[test]
    fn hook_logs_panic_info_and_backtrace() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        log::set_boxed_logger(Box::new(Collector(collected.clone()))).unwrap();
        log::set_max_level(log::LevelFilter::Trace);

        init_hook();
        let handle = std::thread::spawn(|| panic!("panic hook test"));
        let _ = handle.join();

        let logs = collected.lock().unwrap();
        assert!(
            logs.iter()
                .any(|entry| entry.contains("[panic]") && entry.contains("panic hook test")),
            "panic 信息未写入日志: {logs:?}"
        );
        assert!(
            logs.iter().any(|entry| entry.contains("[panic] backtrace")),
            "backtrace 未写入日志: {logs:?}"
        );
    }
}
