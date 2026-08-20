<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import { Button } from "$components/ui/button";
  import { Skeleton } from "$components/ui/skeleton";
  import { m } from "$libs/i18n/paraglide/messages";
  import { getProject } from "$features/projects";
  import type { Project } from "$features/projects";
  import ProjectForm from "./ProjectForm.svelte";

  let project = $state<Project | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  // 路由为 /projects/edit/[id]，id 为 uuid 段（无需 urn:uuid: 前缀，后端 sanitize 兼容）
  const id = $derived($page.params.id as string | undefined);
  const identifier = $derived.by(() => {
    const raw = id;
    if (!raw) return null;
    // 若已含 urn 前缀则直接用，否则拼接
    if (raw.startsWith("urn:uuid:")) return raw;
    return `urn:uuid:${raw}`;
  });

  async function load() {
    if (!identifier) {
      loadError = m.projects_edit_load_failed();
      loading = false;
      return;
    }
    loading = true;
    loadError = null;
    try {
      const data = await getProject(identifier);
      if (data) {
        project = data;
      } else {
        loadError = m.projects_edit_load_failed();
      }
    } catch {
      loadError = m.projects_edit_load_failed();
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
  });

  // 当 id 变化时重载（Svelte 5 响应）
  $effect(() => {
    if (id) void load();
  });
</script>

{#if loading}
  <div class="flex h-full w-full items-center justify-center p-6">
    <div class="w-full max-w-5xl space-y-3">
      <Skeleton class="h-7 w-20" />
      <Skeleton class="h-48 w-full" />
    </div>
  </div>
{:else if loadError}
  <div class="flex h-full w-full flex-col items-center justify-center gap-3 p-6">
    <p class="text-sm text-destructive">{loadError}</p>
    <Button variant="outline" size="sm" onclick={() => void goto(resolve("/projects"))}>{m.projects_create_back()}</Button>
  </div>
{:else if project}
  <ProjectForm mode="edit" initial={project} />
{:else}
  <div class="flex h-full w-full items-center justify-center p-6">
    <p class="text-sm text-muted-foreground">{m.projects_edit_load_failed()}</p>
  </div>
{/if}
