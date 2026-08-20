/**
 * 浮层模块统一出口：toast 能力的唯一引用点。
 * 消费方经 `$libs/overlay` 使用，不直接依赖 svelte-sonner（与 ipc/logger 同模式）；
 * 后续可在此集中统一默认参数（position/duration 等）。
 * 确认对话框（ConfirmDialog）为 props 驱动组件，调用方局部定义使用，不在此维护状态。
 */

export { toast } from "svelte-sonner";
