import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { paraglideVitePlugin } from "@inlang/paraglide-js";
import tauriConf from "./src-tauri/tauri.conf.json" with { type: "json" };
import pkg from "./package.json" with { type: "json" };

// TAURI_DEV_HOST：远程/移动端调试时传入目标主机地址
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(() => ({
  plugins: [
    // Tailwind CSS v4：Vite 插件编译（无需 postcss 配置）
    tailwindcss(),
    sveltekit(),
    // 国际化：编译 messages 生成 paraglide 运行时（src/libs/i18n/paraglide）
    paraglideVitePlugin({
      project: "./src/libs/i18n/project.inlang",
      outdir: "./src/libs/i18n/paraglide",
      emitTsDeclarations: true,
      // locale 经 localStorage 跨 reload 存活（首帧渲染前即可解析正确语言）；
      // config.json 为持久真相源，changeLocale 双写保持两者一致，
      // 外部改动 config 导致失同步时 syncLocale 以 config 为准自愈（reload 一次）
      strategy: ["localStorage", "baseLocale"],
    }),
    // 守卫 Svelte 虚拟 CSS 模块：vite-plugin-svelte 在 HMR 竞态下 load 返回 undefined 时，
    // Vite 会回退读取原始 .svelte 源码，被 @tailwindcss/vite 当作 CSS 解析报错
    // （Invalid declaration: onMount）。此处兜底返回空 CSS，阻断回退路径；
    // 上游修复源码文件竞态后可移除（见 sveltejs/vite-plugin-svelte#1333/#1325）
    {
      name: "guard-svelte-virtual-css",
      load(id) {
        if (/[?&]svelte&type=style&lang\.css$/.test(id)) return "";
      },
    },
  ],
  // 经 define 整体注入配置对象，供前端静态引用（build 期替换）：
  // __APP_TAURI_CONF__ 为整份 tauri.conf.json、__APP_PKG__ 为整份 package.json；
  // 消费方按需取属性（如 __APP_TAURI_CONF__.app.windows[0].title）；
  // 注意 server.watch 忽略了 src-tauri，改配置后需重启 dev 生效
  define: {
    __APP_TAURI_CONF__: JSON.stringify(tauriConf),
    __APP_PKG__: JSON.stringify(pkg),
  },
  clearScreen: false,
  server: {
    // dev 端口固定 1420，与 tauri.conf.json 的 devUrl 及 CSP 保持一致
    port: 1420,
    strictPort: true,
    host: host || false,
    // 通过 TAURI_DEV_HOST 访问时，HMR websocket 单独使用 1421 端口
    ws: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 避免监听 Rust 后端目录与构建产物引发无谓重启；
      // target/ 为 workspace 根产物目录（不在 src-tauri/ 内），Windows 上 watch 被
      // cargo 锁定的构建脚本 exe 会报 EBUSY 崩溃（Linux 仅浪费资源）
      ignored: ["**/src-tauri/**", "**/target/**"],
    },
  },
}));
