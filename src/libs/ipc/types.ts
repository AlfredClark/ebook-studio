/** IPC 统一响应包裹（与 Rust 侧 cores/response.rs 对齐） */
export interface Response<T> {
  code: number; // 业务码：0 成功，非 0 失败
  message: string; // 提示信息
  data: T | null; // 业务数据；失败时为 null
}

/** 成功业务码 */
export const CODE_OK = 0;

/** 系统级配置快照（与 Rust 侧 cores/config.rs 的 SystemConfig 对齐） */
export interface SystemConfig {
  locale: string; // 界面语言存储模式值（"system" 跟随系统，或具体语言标签）
  autostart: boolean; // 开机自启开关
  tray: boolean; // 系统托盘开关
  notification: boolean; // 系统通知开关
  windowState: boolean; // 窗口状态记忆开关
}
