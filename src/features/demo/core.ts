/**
 * 演示业务：问候语生成（与后端 `src-tauri/src/features/demo.rs` 镜像）。
 *
 * 业务逻辑层约定：features 可直接调 `invokeCommand`（等同后端 commands+features
 * 合并层，不复刻 commands 薄层）；失败返回 null，调用方 `?? 兜底`。
 */

import { invokeCommand } from "$libs/ipc";

/**
 * 生成本地化问候语（后端命令 `greet`，文案经 rust-i18n 本地化）。
 * 前端调用示例：`invokeCommand("greet", { name })`
 * @param name 用户输入的名称
 * @returns 问候语；调用失败时返回 null
 */
export async function greet(name: string): Promise<string | null> {
  return invokeCommand<string>("greet", { name });
}
