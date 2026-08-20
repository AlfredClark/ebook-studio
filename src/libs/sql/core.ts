/**
 * SQL 能力层：封装 tauri-plugin-sql（后端仅插件壳，schema 由本模块幂等建表）。
 *
 * 数据库存储于 AppData 目录（与 config.json 同目录）：插件对相对路径固定解析到
 * AppConfig 目录，故此处经 appDataDir() 拼绝对路径传入（绝对路径透传）。
 * 当前仅承载演示（greet 记录落库与查询）；业务表在此追加建表语句与类型化函数。
 * 用法：先 `initSql()` 一次（幂等），再调用 insert/list。
 */

import { appDataDir, join } from "@tauri-apps/api/path";
import Database from "@tauri-apps/plugin-sql";
import { error } from "$libs/logger";
import type { GreetLog } from "./types";

/** 数据库文件名（位于 AppData 目录，与 config.json 同目录） */
const DB_FILE_NAME = "app.db";

// language=SQL format=false
/** 建表语句（幂等；演示表 greet_log，模板初始化时可随 demo 模块一并移除） */
const SCHEMA_SQL = `
CREATE TABLE IF NOT EXISTS greet_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  greeting TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
)`;

/** 模块级单例连接（ESM 仅加载一次）；null 表示未初始化或初始化失败 */
let db: Database | null = null;

/**
 * 初始化 SQL 能力：加载 AppData 下的数据库并幂等建表（重复调用直接复用）。
 * @returns 是否就绪；失败时写入日志并返回 false（调用方自行兜底）
 */
export async function initSql(): Promise<boolean> {
  if (db) return true;
  try {
    const dir = await appDataDir();
    const path = await join(dir, DB_FILE_NAME);
    const instance = await Database.load(`sqlite:${path}`);
    await instance.execute(SCHEMA_SQL);
    db = instance;
    return true;
  } catch (err) {
    void error(`[sql] init 失败: ${err}`).catch(() => {});
    return false;
  }
}

/**
 * 记录一条 greet 日志（演示用）。
 * @param name 用户输入的名称
 * @param greeting 生成的问候语
 * @returns 是否写入成功
 */
export async function insertGreetLog(name: string, greeting: string): Promise<boolean> {
  if (!(await initSql())) return false;
  try {
    await db!.execute("INSERT INTO greet_log (name, greeting) VALUES ($1, $2)", [name, greeting]);
    return true;
  } catch (err) {
    void error(`[sql] insert greet_log 失败: ${err}`).catch(() => {});
    return false;
  }
}

/**
 * 查询 greet 记录列表（按 id 倒序，最新的在前）。
 * @returns 记录列表；失败时返回 null
 */
export async function listGreetLogs(): Promise<GreetLog[] | null> {
  if (!(await initSql())) return null;
  try {
    return await db!.select<GreetLog[]>("SELECT id, name, greeting, created_at FROM greet_log ORDER BY id DESC");
  } catch (err) {
    void error(`[sql] select greet_log 失败: ${err}`).catch(() => {});
    return null;
  }
}
