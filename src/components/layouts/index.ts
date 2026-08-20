import type { Component, Snippet } from "svelte";
import type { LayoutName } from "$libs/stores";
import Dashboard from "./Dashboard.svelte";
import Default from "./Default.svelte";

/** 布局注册表：新增布局在此追加，键名即持久化的布局偏好值 */
export const layouts: Record<LayoutName, Component<{ children: Snippet }>> = {
  default: Default,
  dashboard: Dashboard,
};
