<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { onDestroy, onMount } from "svelte";
  import LayoutContainer from "$components/layouts/LayoutContainer.svelte";
  import { EVENT_MENU_NAVIGATE, listenEvent, type MenuPage } from "$libs/events";

  let { children } = $props();

  // 应用菜单（macOS）导航事件：Rust 侧经 events.rs emit menu:navigate（payload 页面标识）→ 切换对应路由
  let unlisten: (() => void) | undefined;
  let disposed = false;
  onMount(() => {
    void listenEvent<MenuPage>(EVENT_MENU_NAVIGATE, (payload) => {
      const pages = { home: "/", settings: "/settings", about: "/about" } as const;
      // 经 $app/paths 的 resolve 包一层：满足内部导航校验（eslint 规则要求）且自动加 base 前缀
      void goto(resolve(pages[payload] ?? "/"));
    })
      .then((fn) => {
        // listen 异步 resolve：若组件已销毁，立即清理而非留存泄漏
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => {});
  });
  onDestroy(() => {
    disposed = true;
    unlisten?.();
  });
</script>

<LayoutContainer>
  {@render children()}
</LayoutContainer>
