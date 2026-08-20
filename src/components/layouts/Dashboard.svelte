<script lang="ts">
  import { page } from "$app/state";
  import AppSidebar from "$components/layouts/parts/AppSidebar.svelte";
  import SidebarTrigger from "$components/layouts/parts/SidebarTrigger.svelte";
  import WindowControl from "$components/layouts/parts/WindowControl.svelte";
  import { defaultNavItems, getActiveNavItem } from "$components/layouts/parts/nav-items";
  import * as Sidebar from "$components/ui/sidebar";
  let { children } = $props();

  // 顶栏标题与当前路由绑定：匹配导航项文案（m.nav_*），未知路径回退首页标题
  // 前缀匹配使 /projects/create 仍显示 Projects 标题
  const pageTitle = $derived(getActiveNavItem(page.url.pathname)?.label() ?? defaultNavItems[0]?.label());
</script>

<Sidebar.Provider open={false} class="h-svh overflow-hidden select-none">
  <AppSidebar />
  <Sidebar.Inset>
    <header class="flex h-10 shrink-0 items-center border-b select-none">
      <SidebarTrigger />
      <div class="flex min-w-0 flex-1 items-center px-4 text-sm" data-tauri-drag-region>
        <span class="truncate">{pageTitle}</span>
      </div>
      <WindowControl />
    </header>
    <main class="flex-1 overflow-y-auto">
      {@render children()}
    </main>
  </Sidebar.Inset>
</Sidebar.Provider>
