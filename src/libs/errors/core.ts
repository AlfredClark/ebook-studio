import { error } from "$libs/logger";

/** 日志前缀：与后端 cores/panic.rs 的 [panic] 风格对齐 */
const PREFIX = "[error]";

/** 归一化异常对象：Error 取 name/message/stack，其余转字符串 */
function formatError(target: unknown): string {
  if (target instanceof Error) {
    return `${target.name}: ${target.message}\n${target.stack ?? "(no stack)"}`;
  }
  return String(target);
}

/** 防重入守卫：日志写入失败不得再次进入错误事件处理器 */
let handling = false;

/**
 * 安全写错误日志：写入失败静默忽略（plugin-log 走 IPC，非 Tauri 环境会 reject；
 * 若不 catch，rejection 会触发 unhandledrejection → 处理器再写日志 → 无限循环）。
 * 防重入守卫经 Promise 链释放：标志保持到日志写入尝试结束，异步期间不重入。
 */
function logSafe(message: string): void {
  if (handling) return;
  handling = true;
  error(`${PREFIX} ${message}`)
    .catch(() => {})
    .finally(() => {
      handling = false;
    });
}

/**
 * 注册全局异常拦截：未捕获异常（含资源加载失败）+ 未处理 Promise rejection。
 * 应于应用启动最早时机调用（hooks.client.ts 模块作用域）。
 */
export function initErrorHooks(): void {
  // capture 阶段监听：资源加载错误不冒泡，须在捕获阶段捕获
  window.addEventListener(
    "error",
    (event) => {
      if (event.message) {
        logSafe(`uncaught: ${event.message} at ${event.filename}:${event.lineno}\n${formatError(event.error)}`);
      } else {
        const target = event.target as HTMLElement | null;
        logSafe(`resource load failed: <${target?.tagName.toLowerCase() ?? "unknown"}>`);
      }
    },
    true,
  );
  window.addEventListener("unhandledrejection", (event) => {
    logSafe(`unhandled rejection: ${formatError(event.reason)}`);
  });
}

/** 记录 SvelteKit 渲染期错误（hooks.client.ts 的 handleError 调用）。 */
export function logSvelteKitError(input: { error: unknown; status: number; message: string }): void {
  logSafe(`sveltekit error (status=${input.status}): ${input.message || formatError(input.error)}`);
}

/** 记录 svelte:boundary 捕获的组件渲染错误（+layout.svelte 调用；onerror 首参为裸 error）。 */
export function logBoundaryError(error: unknown): void {
  logSafe(`boundary error: ${formatError(error)}`);
}
