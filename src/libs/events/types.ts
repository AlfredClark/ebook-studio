/** 前端事件契约：事件名常量 + payload 类型（与后端 `src-tauri/src/cores/events.rs` 镜像对齐）。 */

/** 菜单导航事件（payload 为 MenuPage，后端 emit） */
export const EVENT_MENU_NAVIGATE = "menu:navigate";

/** 退出请求事件（托盘退出菜单/Ctrl+Q 经此事件交前端按 closeBehavior 编排退出流程；payload 为空） */
export const EVENT_REQUEST_EXIT = "app:request-exit";

/** 菜单导航页面标识（与后端 MenuPage 枚举 serde lowercase 序列化对齐） */
export type MenuPage = "home" | "clean" | "settings" | "about";
