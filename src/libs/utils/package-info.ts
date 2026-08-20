/**
 * package.json 注入信息工具：经 vite define 的 __APP_PKG__ 提取展示字段。
 * author 兼容字符串与对象两种形态，统一归一为展示名。
 */

/**
 * 提取应用作者展示名（兼容 __APP_PKG__.author 的字符串与对象两种形态）。
 * @returns 作者展示名；缺失时返回空字符串
 */
export function getAppAuthor(): string {
  const author = __APP_PKG__.author as string | { name?: string };
  return typeof author === "string" ? author : (author.name ?? "");
}
