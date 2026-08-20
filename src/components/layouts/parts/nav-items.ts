/**
 * 布局导航部件的数据与类型：默认导航项 + NavItem 契约。
 * 数据/UI 同域内聚（原 libs/navigation 并入）——唯一消费者为本目录 TabsNavBar。
 */

import { FlaskConical, House, Info, Settings } from "@lucide/svelte";
import type { Component } from "svelte";
import type { Pathname } from "$app/types";
import { m } from "$libs/i18n/paraglide/messages";

/** 导航项：label 为 Paraglide 消息函数（运行期取当前语言文案），href 为内部路由路径 */
export interface NavItem {
  label: () => string;
  href: Pathname;
  icon?: Component<{ class?: string }>;
}

/** 默认导航项：新增页面在此追加（label 一律经 m.xxx() 取，不硬编码文案） */
export const defaultNavItems: NavItem[] = [
  { label: m.nav_home, href: "/", icon: House },
  // demo 演示页：模板初始化时整体移除（连同 components/pages/demo、features/demo 及后端 demo 模块）
  { label: m.nav_demo, href: "/demo", icon: FlaskConical },
  { label: m.nav_settings, href: "/settings", icon: Settings },
  { label: m.nav_about, href: "/about", icon: Info },
];
