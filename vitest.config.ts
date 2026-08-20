import { defineConfig } from "vitest/config";
import tauriConf from "./src-tauri/tauri.conf.json" with { type: "json" };
import pkg from "./package.json" with { type: "json" };

// Vitest 独立配置（不挂 sveltekit 插件，纯 node 环境）：
// 与 vite.config.ts 相同的 define 注入（__APP_PKG__ 供 package-info 测试消费）；
// include 显式限定当前测试域（libs/utils 纯工具），新增测试域时在此扩展
export default defineConfig({
  define: {
    __APP_TAURI_CONF__: JSON.stringify(tauriConf),
    __APP_PKG__: JSON.stringify(pkg),
  },
  test: {
    include: ["src/libs/utils/**/*.test.ts"],
    environment: "node",
  },
});
