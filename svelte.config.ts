import type { Config } from "@sveltejs/kit";
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const config: Config = {
  // 使用vite的预处理器
  preprocess: vitePreprocess(),
  kit: {
    // 静态适配 + fallback = SPA 模式：所有路由回退到 index.html，
    // 由前端路由接管，适配 Tauri 本地文件加载场景
    adapter: adapter({
      fallback: "index.html",
    }),
    // 单入口打包：消除 module preload + 动态 import 瀑布的串行协议请求链，
    // 缩短 Tauri 首帧白屏（JS 仍外链，不影响 CSP；模板应用路由量级小，无拆包收益）
    output: {
      bundleStrategy: "single",
    },
    // 自定义别名：$libs 指向 src/libs
    alias: {
      $libs: "./src/libs",
      $components: "./src/components",
      $features: "./src/features",
      $styles: "./src/styles",
    },
  },
};

export default config;
