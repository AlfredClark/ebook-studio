/**
 * cn() 单元测试：shadcn 工具（clsx 条件合并 + tailwind-merge 冲突解析）。
 * 纯函数无前后端交互，属模板单测约定覆盖范围。
 */

import { describe, expect, it } from "vitest";
import { cn } from "./shadcn";

describe("cn", () => {
  it("合并多个字符串类名", () => {
    expect(cn("a", "b", "c")).toBe("a b c");
  });

  it("过滤 falsy 条件值", () => {
    const isActive = false;
    const isHidden = null;
    expect(cn("a", isActive && "b", isHidden, undefined, 0, "", "c")).toBe("a c");
  });

  it("支持对象与数组形态的条件类名", () => {
    expect(cn({ a: true, b: false }, ["c", { d: true }])).toBe("a c d");
  });

  it("tailwind-merge 解析同组冲突类，后者生效", () => {
    expect(cn("px-2", "px-4")).toBe("px-4");
    expect(cn("bg-red-500", "bg-blue-500")).toBe("bg-blue-500");
  });

  it("保留不同组类名", () => {
    expect(cn("px-2 py-1", "rounded-md")).toBe("px-2 py-1 rounded-md");
  });
});
