/**
 * SQL 数据层类型契约：与前端操作的表结构对应（表结构见 core.ts 的建表语句）。
 */

/** greet 记录：greet_log 表的一行（created_at 为 SQLite datetime 本地时间字符串） */
export interface GreetLog {
  id: number;
  name: string;
  greeting: string;
  created_at: string;
}
