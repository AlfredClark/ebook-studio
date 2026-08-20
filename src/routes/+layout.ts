// 关闭 SSR：Tauri 通过本地文件/自建协议加载页面，无服务器环境，
// 页面必须纯客户端渲染（SPA），构建产物为静态文件
export const ssr = false;
