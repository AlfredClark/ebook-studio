/**
 * SQL 数据层统一出口：封装 tauri-plugin-sql 的加载、建表与类型化读写。
 * 业务表函数与类型在此追加导出。
 */

export { getDb, initSql } from "./core";
