import { initErrorHooks, logSvelteKitError } from "$libs/errors";
import type { HandleClientError } from "@sveltejs/kit";

// 模块作用域注册：先于任何渲染捕获 window 异常与未处理 rejection
initErrorHooks();

/** SvelteKit 渲染期错误：写入共享日志链路（与后端 panic hook 同链） */
export const handleError: HandleClientError = ({ error, status, message }) => {
  logSvelteKitError({ error, status, message });
};
