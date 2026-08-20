/**
 * getAppAuthor() 单元测试：package.json 注入信息工具。
 * 已知边界：__APP_PKG__ 经 define 静态注入真实 package.json（author 为字符串形态），
 * 对象形态分支无法在测试环境注入覆盖（保持生产代码不动，仅测字符串分支）。
 */

import { describe, expect, it } from "vitest";
import { getAppAuthor } from "./package-info";

describe("getAppAuthor", () => {
  it("返回 package.json 的 author 展示名（字符串形态原样返回）", () => {
    expect(getAppAuthor()).toBe("Alfred Clark");
  });

  it("返回值为非空字符串", () => {
    const author = getAppAuthor();
    expect(typeof author).toBe("string");
    expect(author.length).toBeGreaterThan(0);
  });
});
