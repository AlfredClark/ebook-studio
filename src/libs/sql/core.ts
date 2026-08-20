/**
 * SQL 能力层：封装 tauri-plugin-sql（后端仅插件壳，schema 由本模块幂等建表）。
 *
 * 数据库存储于 AppData 目录（与 config.json 同目录）：插件对相对路径固定解析到
 * AppConfig 目录，故此处经 appDataDir() 拼绝对路径传入（绝对路径透传）。
 * 业务表在此追加建表语句与类型化函数。
 * 用法：先 `initSql()` 一次（幂等），再调用业务读写函数。
 */

import { appDataDir, join } from "@tauri-apps/api/path";
import Database from "@tauri-apps/plugin-sql";
import { error } from "$libs/logger";

/** 数据库文件名（位于 AppData 目录，与 config.json 同目录） */
const DB_FILE_NAME = "app.db";

// language=SQL format=false
/** 建表语句（幂等；初始化时为空，后续业务表在此追加） */
const SCHEMA_SQL = "";

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
    if (SCHEMA_SQL.trim()) {
      await instance.execute(SCHEMA_SQL);
    }
    db = instance;
    return true;
  } catch (err) {
    void error(`[sql] init 失败: ${err}`).catch(() => {});
    return false;
  }
}

/**
 * 获取数据库实例（需先 initSql 成功）。
 * @returns Database 实例或 null
 */
export function getDb(): Database | null {
  return db;
}
