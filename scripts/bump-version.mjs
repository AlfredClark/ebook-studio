/**
 * 应用版本脚本：按 patch / minor / major 三个等级递增版本号，或直接指定版本号，
 * 并同步写回三处版本真相源（package.json / src-tauri/Cargo.toml / src-tauri/tauri.conf.json）。
 *
 * 用法：
 *   node scripts/bump-version.mjs patch   # 0.1.0 → 0.1.1
 *   node scripts/bump-version.mjs minor   # 0.1.0 → 0.2.0
 *   node scripts/bump-version.mjs major   # 0.1.0 → 1.0.0
 *   node scripts/bump-version.mjs 0.3.0   # 直接设为指定版本
 *
 * 说明：
 * - 仅校验格式，不校验目标版本高低（回退场景由开发者自行把控）
 * - Cargo.lock 中本包版本条目由下次 cargo check/build 自动刷新，脚本不处理
 * - 不执行 git 操作（提交/打 tag 由开发者手动完成）
 */

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

const PACKAGE_JSON = join(rootDir, "package.json");
const CARGO_TOML = join(rootDir, "src-tauri", "Cargo.toml");
const TAURI_CONF = join(rootDir, "src-tauri", "tauri.conf.json");

/** 合法等级集合 */
const LEVELS = ["patch", "minor", "major"];

/** 版本号正则：纯数字三段式（模板版本均无 pre-release，不支持带后缀形态） */
const VERSION_RE = /^(\d+)\.(\d+)\.(\d+)$/;

/**
 * 解析版本号；非纯数字三段式时抛错退出。
 * @param value 版本字符串
 * @returns [major, minor, patch]
 */
function parseVersion(value) {
  const match = VERSION_RE.exec(value);
  if (!match) {
    console.error(`[bump-version] invalid version "${value}", expected x.y.z`);
    process.exit(1);
  }
  return match.slice(1).map(Number);
}

/**
 * 按等级递增版本号。
 * @param parts
 * @param level 提升等级
 * @returns *[] [major, minor, patch]
 */
function bumpVersion(parts, level) {
  const [major, minor, patch] = parts;
  switch (level) {
    case "patch":
      return [major, minor, patch + 1];
    case "minor":
      return [major, minor + 1, 0];
    case "major":
      return [major + 1, 0, 0];
  }
}

/**
 * 写入 package.json：解析后仅改顶层 version，JSON.stringify 重写保持缩进/键序，末尾补换行。
 */
function writePackageJson(next) {
  const pkg = JSON.parse(readFileSync(PACKAGE_JSON, "utf8"));
  pkg.version = next;
  writeFileSync(PACKAGE_JSON, `${JSON.stringify(pkg, null, 2)}\n`);
}

/**
 * 写入 Cargo.toml：仅替换首个匹配的 `version = "x.y.z"` 行（[package] 段），
 * 不误伤 [dependencies] 中形如 `version = "2"` 的依赖版本行。
 */
function writeCargoToml(next) {
  const content = readFileSync(CARGO_TOML, "utf8");
  const updated = content.replace(/^(version = "\d+\.\d+\.\d+")$/m, `version = "${next}"`);
  if (updated === content) {
    console.error(`[bump-version] ${CARGO_TOML}: package version line not found`);
    process.exit(1);
  }
  writeFileSync(CARGO_TOML, updated);
}

/**
 * 写入 tauri.conf.json：仅替换顶层 "version" 键（首个匹配）。
 */
function writeTauriConf(next) {
  const content = readFileSync(TAURI_CONF, "utf8");
  const updated = content.replace(/"version": "\d+\.\d+\.\d+"/, `"version": "${next}"`);
  if (updated === content) {
    console.error(`[bump-version] ${TAURI_CONF}: version key not found`);
    process.exit(1);
  }
  writeFileSync(TAURI_CONF, updated);
}

function main() {
  const arg = process.argv[2];
  let next;
  let mode;
  if (LEVELS.includes(arg)) {
    const current = JSON.parse(readFileSync(PACKAGE_JSON, "utf8")).version;
    next = bumpVersion(parseVersion(current), arg).join(".");
    mode = arg;
  } else if (VERSION_RE.test(arg)) {
    // 直接指定版本号：仅校验格式，不校验高低（回退场景由开发者自行把控）
    next = arg;
    mode = "set";
  } else {
    console.error(`[bump-version] invalid argument "${arg}", expected a level (${LEVELS.join(", ")}) or a version (x.y.z)`);
    process.exit(1);
  }

  const current = JSON.parse(readFileSync(PACKAGE_JSON, "utf8")).version;

  writePackageJson(next);
  writeCargoToml(next);
  writeTauriConf(next);
  console.log(`[bump-version] ${current} → ${next} (${mode})`);
}

main();
