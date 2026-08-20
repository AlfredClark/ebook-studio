<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import WindowControl from "$components/layouts/parts/WindowControl.svelte";
  import { Button } from "$components/ui/button";
  import * as Breadcrumb from "$components/ui/breadcrumb";
  import { WORKSPACE_STAGES, getActiveWorkspaceStage } from "$components/pages/workspace/stages";
  import { m } from "$libs/i18n/paraglide/messages";

  let { children } = $props();

  const workspaceId = $derived(page.params.id as string | undefined);
  const activeStage = $derived(getActiveWorkspaceStage(page.url.pathname));

  function handleBack() {
    void goto(resolve("/projects"));
  }
</script>

<div class="flex h-screen w-screen flex-col overflow-hidden">
  <header class="flex h-9 shrink-0 items-center gap-1 border-b px-1 select-none" data-tauri-drag-region>
    <Button variant="ghost" size="icon" class="size-7 shrink-0" onclick={handleBack} aria-label={m.workspace_back()}>
      <ArrowLeftIcon class="size-4" />
    </Button>

    {#if workspaceId}
      <Breadcrumb.Root class="shrink">
        <Breadcrumb.List>
          {#each WORKSPACE_STAGES as stage, i (stage.value)}
            <Breadcrumb.Item>
              {#if activeStage === stage.value}
                <Breadcrumb.Page>{stage.label()}</Breadcrumb.Page>
              {:else}
                <Breadcrumb.Link href={resolve(stage.href(workspaceId))}>{stage.label()}</Breadcrumb.Link>
              {/if}
            </Breadcrumb.Item>
            {#if i < WORKSPACE_STAGES.length - 1}
              <Breadcrumb.Separator />
            {/if}
          {/each}
        </Breadcrumb.List>
      </Breadcrumb.Root>
    {:else}
      <div class="flex-1"></div>
    {/if}

    <div class="flex-1" data-tauri-drag-region></div>
    <WindowControl />
  </header>

  <main class="flex flex-1 overflow-hidden">
    {@render children()}
  </main>
</div>
