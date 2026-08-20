/** 主题名列表：与 themes/*.css 一一对应，单一真相源（新增主题在此追加，并同步 index.css 与 label 映射）。
 *  完整 token 主题：neutral（基底，含 :root 首帧兜底）/ stone / zinc / mauve / olive / mist / taupe；
 *  局部覆盖主题基于 neutral 基底仅覆盖差异 token（data-theme 未覆盖 token 回落基底值）。 */
export const themeNames = ["neutral", "stone", "zinc", "mauve", "olive", "mist", "taupe"] as const;

/** 主题名类型：由 themeNames 推导（data-theme 属性值即主题名；neutral 为基底主题） */
export type ThemeName = (typeof themeNames)[number];
