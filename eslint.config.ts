import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import svelte from "eslint-plugin-svelte";
import prettier from "eslint-config-prettier";
import svelteConfig from "./svelte.config.ts";
import { defineConfig, globalIgnores } from "eslint/config";

// vite define 注入的全局常量（与 vite.config.ts 的 define 保持一致）
const viteDefineGlobals = {
  __APP_TAURI_CONF__: "readonly",
  __APP_PKG__: "readonly",
};

export default defineConfig([
  {
    // 基础规则集：JS 推荐 + TS 推荐 + Svelte 推荐 + 与 prettier 兼容
    extends: [
      js.configs.recommended,
      ...tseslint.configs.recommended,
      ...svelte.configs.recommended,
      ...svelte.configs.prettier,
    ],
  },
  // 全局忽略：生成目录、构建产物与 Rust 后端
  globalIgnores([
    "build/",
    ".svelte-kit/",
    "package/",
    "node_modules/",
    "src-tauri/",
    "target/",
    "src/libs/i18n/paraglide/",
    "src/libs/i18n/project.inlang/",
    "**/.*",
  ]),
  {
    // 浏览器环境下的 TS 文件
    files: ["src/**/*.ts"],
    languageOptions: {
      globals: { ...globals.browser, ...viteDefineGlobals },
    },
  },
  {
    // Svelte 组件与 runes 模块（.svelte.ts）：使用 svelte 解析器 + TS + 浏览器 globals
    files: ["src/**/*.svelte", "src/**/*.svelte.ts"],
    languageOptions: {
      globals: { ...globals.browser, ...viteDefineGlobals },
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".svelte", ".svelte.ts"],
        svelteConfig,
      },
    },
  },
  {
    // shadcn-svelte 生成组件豁免：按钮类组件 href 为动态绑定（外链/禁用态），
    // 不适用 SPA 的 resolve() 导航约束（svelte/no-navigation-without-resolve 误报）
    files: ["src/components/ui/**/*.svelte"],
    rules: {
      "svelte/no-navigation-without-resolve": "off",
    },
  },
  {
    // Node 环境下的构建配置文件与脚本
    files: ["*.config.ts", "eslint.config.ts", "scripts/**/*.mjs"],
    languageOptions: { globals: globals.node },
  },
  // 置于最后，关闭与 prettier 冲突的规则
  prettier,
]);
