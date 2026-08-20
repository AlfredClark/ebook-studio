<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Check } from "@lucide/svelte";
  import { Badge } from "$components/ui/badge";
  import { Button } from "$components/ui/button";
  import { Label } from "$components/ui/label";
  import GithubIcon from "$components/widgets/icons/GithubIcon.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { getAppAuthor } from "$libs/utils";
  import { checkUpdate, installPendingUpdate, relaunchApp, update } from "$libs/updater";

  const appAuthor = getAppAuthor();
  const homepage = __APP_PKG__.homepage ?? "";

  /** 主页跳转：经系统默认浏览器打开，失败静默（不阻断页面） */
  async function openHomepage() {
    if (!homepage) return;
    try {
      await openUrl(homepage);
    } catch {
      // 打开失败静默忽略
    }
  }
</script>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <Label>{m.about_app_name()}</Label>
  <span class="text-sm text-muted-foreground">{__APP_PKG__.name}</span>
</div>

<div class="px-4 py-4">
  <div class="flex items-center justify-between gap-4">
    <Label>{m.about_app_version()}</Label>
    <div class="flex items-center gap-2">
      <span class="text-sm text-muted-foreground">{__APP_PKG__.version}</span>
      {#if update.status === "idle" || update.status === "error"}
        <Button variant="outline" size="sm" onclick={() => void checkUpdate()}>
          {m.about_app_check_update()}
        </Button>
      {:else if update.status === "upToDate"}
        <Badge variant="secondary">
          <Check />
          {m.about_app_up_to_date()}
        </Badge>
      {:else if update.status === "installed"}
        <Badge variant="secondary">{m.about_app_installed()}</Badge>
        <Button variant="outline" size="sm" onclick={() => void relaunchApp()}>
          {m.about_app_restart()}
        </Button>
      {:else if update.status === "checking"}
        <Button variant="outline" size="sm" disabled>{m.about_app_checking()}</Button>
      {:else if update.status === "available"}
        <Button variant="outline" size="sm" onclick={() => void installPendingUpdate()}>
          {m.about_app_update({ version: update.version ?? "" })}
        </Button>
      {:else if update.status === "downloading"}
        <Button variant="outline" size="sm" disabled>
          {m.about_app_downloading({ percent: String(update.percent ?? 0) })}
        </Button>
      {/if}
    </div>
  </div>
  {#if update.status === "error"}
    <p class="mt-2 text-right text-sm text-destructive">{m.about_app_update_failed()}</p>
  {:else if update.status === "downloading"}
    <div class="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-muted">
      <div class="h-full bg-primary transition-all" style="width: {update.percent ?? 0}%"></div>
    </div>
  {/if}
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <Label>{m.about_app_author()}</Label>
  <span class="text-sm text-muted-foreground">{appAuthor}</span>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <Label>{m.about_app_license()}</Label>
  <span class="text-sm text-muted-foreground">{__APP_PKG__.license}</span>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <Label>{m.about_app_description()}</Label>
  <span class="text-sm text-muted-foreground">{m.about_app_desc()}</span>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <Label>{m.about_app_homepage()}</Label>
  <Button variant="outline" size="sm" onclick={openHomepage} disabled={!homepage}>
    <GithubIcon class="size-4" />
    {m.about_app_visit_homepage()}
  </Button>
</div>
