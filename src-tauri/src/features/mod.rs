//! 业务功能模块：新增功能的业务逻辑一律放此处（每功能一个模块、单一职责）。
//!
//! 分层约定：`commands/` 为 IPC 命令薄层（校验参数 → 调 features → 转 `Response<T>`），
//! 本模块仅承载业务逻辑（返回 `AppResult<T>`），不直接构造 `Response`；
//! 系统级能力（配置、日志、i18n 等）复用 `cores/` 的公开接口，不重写。

pub mod build;
pub mod clean;
pub mod inspect;
pub mod package;
pub mod projects;
pub mod split;
