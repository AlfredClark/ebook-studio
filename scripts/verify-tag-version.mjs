/**
 * Release 版本守卫：校验 git tag 与三处版本真相源（package.json / src-tauri/Cargo.toml /
 * src-tauri/tauri.conf.json）互一致，避免 tag 与版本错配导致发布产物/更新清单不一致。
 *
 * 用法（CI 中经 bun 运行，无第三方依赖）：
 *   bun scripts/verify-tag-version.mjs 0.5.0    # 无 v 前缀；一致时正常退出
 *   bun scripts/verify-tag-version.mjs v0.5.0   # 带 v 前缀亦可
 *   bun scripts/verify-tag-version.mjs 0.4.0    # 不一致时退出码 1，阻断发布
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

// 兼容带/不带 v 前缀的 tag（如 0.5.0 或 v0.5.0），统一剥离后比对版本号
const tag = process.argv[2];
const expected = tag?.replace(/^v/, "");
if (!expected || !/^\d+\.\d+\.\d+$/.test(expected)) {
  console.error(`[verify-tag-version] invalid tag "${tag}", expected <version> or v<version>`);
  process.exit(1);
}

const pkg = JSON.parse(readFileSync(join(rootDir, "package.json"), "utf8")).version;
const cargo = readFileSync(join(rootDir, "src-tauri", "Cargo.toml"), "utf8").match(/^version = "([^"]+)"/m)?.[1];
const tauriConf = JSON.parse(readFileSync(join(rootDir, "src-tauri", "tauri.conf.json"), "utf8")).version;

const versions = {
  "package.json": pkg,
  "src-tauri/Cargo.toml": cargo,
  "src-tauri/tauri.conf.json": tauriConf,
};
const mismatches = Object.entries(versions).filter(([, version]) => version !== expected);
if (mismatches.length > 0) {
  console.error(
    `[verify-tag-version] tag "${tag}" mismatch: ${mismatches.map(([file, version]) => `${file}=${version}`).join(", ")}`,
  );
  process.exit(1);
}
console.log(`[verify-tag-version] tag "${tag}" matches all version sources (${expected})`);
