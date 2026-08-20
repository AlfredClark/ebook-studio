<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import * as Sidebar from "$components/ui/sidebar";
  import { m } from "$libs/i18n/paraglide/messages";
  import { getAppAuthor } from "$libs/utils";
  import { defaultNavItems, type NavItem } from "./nav-items";

  // 版权年份取运行期当前年，不硬编码
  const currentYear = new Date().getFullYear();
  const appAuthor = getAppAuthor();

  // 选中态与路由绑定：后退/刷新/直接访问均自动同步；
  // 路径与导航项不匹配时（如 tauri://localhost 无路径段、pathname 为空）回退默认项（首页）
  const activeHref = $derived(
    defaultNavItems.find((item) => item.href === page.url.pathname)?.href ?? defaultNavItems[0]?.href,
  );

  async function handleNavigate(item: NavItem) {
    if (item.href !== page.url.pathname) {
      // 经 $app/paths 的 resolve 包一层：满足内部导航校验（eslint 规则要求）且自动加 base 前缀
      await goto(resolve(item.href));
    }
  }
</script>

<Sidebar.Root variant="inset" collapsible="icon" class="transition-none">
  <Sidebar.Header>
    <div
      class="flex w-full items-center gap-2 py-1 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
    >
      <img src="/icon.png" alt="icon" class="size-5 shrink-0 group-data-[collapsible=icon]:mx-auto" />
      <span class="truncate text-sm font-bolder group-data-[collapsible=icon]:hidden">
        {__APP_TAURI_CONF__.app.windows[0].title}
      </span>
    </div>
  </Sidebar.Header>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each defaultNavItems as item (item.href)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={item.href === activeHref} tooltipContent={item.label()} class="[&_svg]:size-5">
                {#snippet child({ props })}
                  {@const Icon = item.icon}
                  <button {...props} type="button" onclick={() => void handleNavigate(item)}>
                    {#if Icon}
                      <Icon />
                    {/if}
                    <span class="group-data-[collapsible=icon]:hidden">{item.label()}</span>
                  </button>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
  <Sidebar.Footer>
    <div class="w-full flex-col gap-0.5 px-2 py-1 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
      <p class="w-full truncate text-center">{m.footer_copyright({ year: String(currentYear), author: appAuthor })}</p>
    </div>
  </Sidebar.Footer>
</Sidebar.Root>
