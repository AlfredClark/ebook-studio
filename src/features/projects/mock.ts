/**
 * 项目演示数据（EPUB3.3 四字段）。
 */

import type { Project } from "./types";

const now = Date.now();
const day = 24 * 60 * 60 * 1000;

/** 演示用项目列表（8 条，阶梯 modified 便于验证排序/搜索） */
export const mockProjects: Project[] = [
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440001",
    title: "哈利·波特与魔法石",
    language: "zh-CN",
    modified: now - 0 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440002",
    title: "Pride and Prejudice",
    language: "en",
    modified: now - 1 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440003",
    title: "人类简史",
    language: "zh-CN",
    modified: now - 3 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440004",
    title: "Norwegian Wood",
    language: "en",
    modified: now - 7 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440005",
    title: "ノルウェイの森",
    language: "ja",
    modified: now - 10 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440006",
    title: "Le Petit Prince",
    language: "fr",
    modified: now - 14 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440007",
    title: "三体",
    language: "zh-CN",
    modified: now - 21 * day,
  },
  {
    identifier: "urn:uuid:550e8400-e29b-41d4-a716-446655440008",
    title: "To Kill a Mockingbird",
    language: "en",
    modified: now - 30 * day,
  },
];
