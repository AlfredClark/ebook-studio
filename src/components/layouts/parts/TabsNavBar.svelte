<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { Tabs, TabsList, TabsTrigger } from "$components/ui/tabs";
  import { defaultNavItems, type NavItem } from "./nav-items";

  /** 导航项：默认使用同目录 nav-items 的 defaultNavItems，布局可传入自定义导航 */
  let { items = defaultNavItems }: { items?: NavItem[] } = $props();

  // 选中态与路由绑定：value 即 href，后退/刷新/直接访问均自动同步；
  // 路径与导航项不匹配时（如 tauri://localhost 无路径段、pathname 为空）回退默认项（首页）
  let activeTab = $derived(items.find((item) => item.href === page.url.pathname)?.href ?? items[0]?.href);

  async function handleValueChange(value: string | undefined) {
    const target = items.find((item) => item.href === value);
    if (target && target.href !== page.url.pathname) {
      // 经 $app/paths 的 resolve 包一层：满足内部导航校验（eslint 规则要求）且自动加 base 前缀
      await goto(resolve(target.href));
    }
  }
</script>

<Tabs value={activeTab} onValueChange={handleValueChange} class="w-fit">
  <TabsList variant="line">
    {#each items as item (item.href)}
      <TabsTrigger value={item.href} class="px-3">
        {@const Icon = item.icon}
        {#if Icon}
          <Icon class="size-4" />
        {/if}
        {item.label()}
      </TabsTrigger>
    {/each}
  </TabsList>
</Tabs>
