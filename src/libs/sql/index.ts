/**
 * SQL 数据层统一出口：封装 tauri-plugin-sql 的加载、建表与类型化读写。
 * 当前仅承载演示（greet 记录）；业务表在此追加函数与类型。
 */

export { initSql, insertGreetLog, listGreetLogs } from "./core";
export type { GreetLog } from "./types";
