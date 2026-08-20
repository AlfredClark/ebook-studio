<script lang="ts">
  // 侧边栏折叠开关（TooltipButton 复合组件）：常态显示 tooltip，文案随折叠状态动态切换（收起↔展开）。
  // 须作为 Sidebar.Provider 的子组件渲染——useSidebar 的 context 仅在其子树内可用，
  // 直接用于 Dashboard（Provider 的父级）会在初始化阶段取不到 context。
  // TooltipButton 经 mergeProps 链式合并委托 onclick，不会覆盖此处 toggle（对比
  // Sidebar.Trigger 的 {...restProps} 展开会覆盖自身 onclick，不能直接包 Tooltip）。
  import PanelLeftIcon from "@lucide/svelte/icons/panel-left";
  import { useSidebar } from "$components/ui/sidebar";
  import TooltipButton from "$components/widgets/overlay/TooltipButton.svelte";
  import { m } from "$libs/i18n/paraglide/messages";

  const sidebar = useSidebar();

  // 动态文案：展开态提示「收起」，收缩态提示「展开」；经 $derived 保持响应式
  const label = $derived(sidebar.state === "expanded" ? m.sidebar_collapse() : m.sidebar_expand());
</script>

<TooltipButton {label} class="ml-2" size="icon-lg" onclick={() => sidebar.toggle()}>
  <PanelLeftIcon />
</TooltipButton>
