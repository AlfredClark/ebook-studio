<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronUpIcon from "@lucide/svelte/icons/chevron-up";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$components/ui/button";
  import { Checkbox } from "$components/ui/checkbox";
  import { Input } from "$components/ui/input";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Skeleton } from "$components/ui/skeleton";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { batchDeleteProjects, deleteProject, listProjects, resolveProjectAsset } from "$features/projects";
  import type { Project, ProjectSortKey } from "$features/projects";
  import { invokeCommand } from "$libs/ipc";

  let projects = $state<Project[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let search = $state("");
  let sortKey = $state<ProjectSortKey>("modifiedDesc");
  let selectedId = $state<string | null>(null);
  let checked = new SvelteSet<string>();
  let showMore = $state(false);
  let coverSrc = $state<string | null>(null);

  const sortOptions: { value: ProjectSortKey; label: () => string }[] = [
    { value: "modifiedDesc", label: m.projects_sort_modifiedDesc },
    { value: "modifiedAsc", label: m.projects_sort_modifiedAsc },
  ];

  const sortLabel = $derived(sortOptions.find((o) => o.value === sortKey)?.label() ?? sortOptions[0].label());
  const checkedCount = $derived(checked.size);
  const selected = $derived(projects.find((p) => p.identifier === selectedId) ?? null);

  const filteredSorted = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = q ? projects.filter((p) => p.title.toLowerCase().includes(q)) : projects.slice();
    list.sort((a, b) => {
      const aMs = a.modifiedMs ?? new Date(a.modified).getTime();
      const bMs = b.modifiedMs ?? new Date(b.modified).getTime();
      if (sortKey === "modifiedAsc") return aMs - bMs;
      return bMs - aMs;
    });
    return list;
  });

  const allChecked = $derived(filteredSorted.length > 0 && filteredSorted.every((p) => checked.has(p.identifier)));
  const indeterminate = $derived(!allChecked && filteredSorted.some((p) => checked.has(p.identifier)));

  async function loadProjects() {
    loading = true;
    loadError = null;
    try {
      const data = await listProjects();
      if (data) {
        projects = data;
        if (selectedId === null && projects.length > 0) {
          selectedId = projects[0].identifier;
        } else if (selectedId && !projects.some((p) => p.identifier === selectedId)) {
          selectedId = projects[0]?.identifier ?? null;
        }
        // 清理已不存在的勾选
        for (const id of Array.from(checked)) {
          if (!projects.some((p) => p.identifier === id)) checked.delete(id);
        }
      } else {
        loadError = m.projects_load_failed();
      }
    } catch {
      loadError = m.projects_load_failed();
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadProjects();
  });

  // 当选中项变更时加载封面预览（通过后端 data URL 规避 asset 协议限制）
  $effect(() => {
    const sel = selected;
    if (!sel?.cover) {
      coverSrc = null;
      return;
    }
    void (async () => {
      const id = sel.identifier;
      const rel = sel.cover!;
      try {
        const abs = await resolveProjectAsset(id, rel);
        if (!abs || selected?.identifier !== id) return;
        const dataUrl = await invokeCommand<string>("read_image_as_data_url", { path: abs });
        if (selected?.identifier === id && dataUrl) {
          coverSrc = dataUrl;
        } else if (selected?.identifier === id) {
          coverSrc = null;
        }
      } catch {
        if (selected?.identifier === id) coverSrc = null;
      }
    })();
  });

  // 选中变更时折叠更多信息
  $effect(() => {
    if (selected) showMore = false;
  });

  function formatDate(value: string | number): string {
    try {
      const d = new Date(value);
      if (isNaN(d.getTime())) return String(value);
      return d.toLocaleString();
    } catch {
      return String(value);
    }
  }

  function toggleChecked(identifier: string, value: boolean) {
    if (value) checked.add(identifier);
    else checked.delete(identifier);
  }

  function handleToggleAll(value: boolean) {
    if (value) {
      for (const p of filteredSorted) checked.add(p.identifier);
    } else {
      for (const p of filteredSorted) checked.delete(p.identifier);
    }
  }

  function handleSelect(identifier: string) {
    if (selectedId === identifier) return;
    selectedId = identifier;
  }

  function handleNewProject() {
    void goto(resolve("/projects/create"));
  }

  async function handleDelete() {
    if (!selected) return;
    const id = selected.identifier;
    const res = await deleteProject(id);
    if (res) {
      toast.success(m.projects_delete_success());
      projects = projects.filter((p) => p.identifier !== id);
      checked.delete(id);
      if (selectedId === id) {
        selectedId = projects[0]?.identifier ?? null;
      }
      coverSrc = null;
    } else {
      toast.error(m.projects_delete_failed());
    }
  }

  async function handleBatchDelete() {
    if (checked.size === 0) return;
    const ids = Array.from(checked);
    const res = await batchDeleteProjects(ids);
    if (res !== null && res > 0) {
      toast.success(m.projects_batch_delete_success({ count: String(res) }));
      projects = projects.filter((p) => !ids.includes(p.identifier));
      checked.clear();
      if (selectedId && ids.includes(selectedId)) {
        selectedId = projects[0]?.identifier ?? null;
      }
      coverSrc = null;
    } else {
      toast.error(m.projects_batch_delete_failed());
    }
  }
</script>

<div class="flex h-full w-full flex-col overflow-hidden">
  <Resizable.PaneGroup direction="horizontal" class="h-full min-h-0 w-full">
    <!-- 左侧：项目列表 -->
    <Resizable.Pane defaultSize={50} minSize={40} maxSize={60} class="flex min-h-0 flex-col overflow-hidden">
      <div class="flex items-center gap-2 border-b p-3">
        <Checkbox
          checked={allChecked}
          {indeterminate}
          onCheckedChange={(v) => handleToggleAll(v === true)}
          aria-label={m.projects_select_all()}
        />

        <Button variant="outline" size="sm" class="h-8 shrink-0 gap-1" onclick={handleNewProject}>
          <PlusIcon class="size-4" />
          {m.projects_new()}
        </Button>

        <Input bind:value={search} placeholder={m.projects_search_placeholder()} class="h-8 flex-1" />

        <Select
          type="single"
          value={sortKey}
          onValueChange={(v) => {
            if (v) sortKey = v as ProjectSortKey;
          }}
        >
          <SelectTrigger class="h-8 w-36 shrink-0">
            {sortLabel}
          </SelectTrigger>
          <SelectContent>
            {#each sortOptions as opt (opt.value)}
              <SelectItem value={opt.value}>{opt.label()}</SelectItem>
            {/each}
          </SelectContent>
        </Select>

        {#if checkedCount > 0}
          <ConfirmDialog
            title={m.projects_batch_delete_confirm_title()}
            message={m.projects_batch_delete_confirm_message({ count: String(checkedCount) })}
            variant="destructive"
            confirmLabel={m.projects_batch_delete({ count: String(checkedCount) })}
            onConfirm={handleBatchDelete}
          >
            {#snippet trigger({ props })}
              <Button variant="destructive" size="sm" class="h-8 shrink-0" {...props}>
                {m.projects_batch_delete({ count: String(checkedCount) })}
              </Button>
            {/snippet}
          </ConfirmDialog>
        {/if}
      </div>

      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ScrollArea class="h-full min-h-0 flex-1">
          <div class="flex flex-col gap-1 p-2">
            {#if loading}
              <div class="space-y-2 p-2">
                {#each [0, 1, 2, 3, 4] as i (i)}
                  <Skeleton class="h-8 w-full" />
                {/each}
              </div>
            {:else if loadError}
              <div class="p-4 text-center">
                <p class="text-sm text-destructive">{loadError}</p>
                <Button variant="outline" size="sm" class="mt-2" onclick={() => void loadProjects()}
                  >{m.boundary_retry()}</Button
                >
              </div>
            {:else if filteredSorted.length === 0}
              <p class="p-4 text-center text-sm text-muted-foreground">
                {search.trim() ? m.projects_empty_filtered() : m.projects_empty()}
              </p>
            {:else}
              {#each filteredSorted as project (project.identifier)}
                <div
                  class="flex items-center gap-2 rounded-md px-2 py-1 transition-colors hover:bg-accent {selectedId ===
                  project.identifier
                    ? 'bg-accent text-accent-foreground'
                    : 'text-foreground'}"
                >
                  <Checkbox
                    checked={checked.has(project.identifier)}
                    onCheckedChange={(v) => toggleChecked(project.identifier, v === true)}
                  />
                  <button
                    type="button"
                    onclick={() => handleSelect(project.identifier)}
                    class="flex min-w-0 flex-1 items-center justify-between gap-3 text-left text-sm"
                  >
                    <span class="min-w-0 flex-1 truncate font-medium">{project.title}</span>
                    <span class="shrink-0 text-xs text-muted-foreground">{project.language}</span>
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        </ScrollArea>
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <!-- 右侧：项目详情 -->
    <Resizable.Pane defaultSize={50} minSize={40} maxSize={60} class="flex min-h-0 flex-col overflow-hidden bg-card">
      {#if !selected}
        <div class="flex h-full items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.projects_detail_empty()}</p>
        </div>
      {:else}
        <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
          <ScrollArea class="min-h-0 flex-1">
            <div class="space-y-4 p-6">
              <!-- 封面 -->
              <div class="flex justify-center">
                {#if coverSrc}
                  <img src={coverSrc} alt="cover" class="aspect-[3/4] w-48 rounded-lg border object-cover shadow-sm" />
                {:else}
                  <div
                    class="flex aspect-[3/4] w-48 items-center justify-center rounded-lg border bg-muted text-xs text-muted-foreground"
                  >
                    {m.projects_detail_no_cover()}
                  </div>
                {/if}
              </div>

              <h2 class="truncate text-center text-base font-semibold" title={selected.title}>{selected.title}</h2>

              {#if selected.creator}
                <p class="text-center text-sm text-muted-foreground">{selected.creator}</p>
              {/if}

              <!-- 基础信息 -->
              <div class="space-y-2 rounded-lg border p-4 text-sm">
                <div class="flex justify-between gap-4">
                  <span class="shrink-0 text-muted-foreground">{m.projects_detail_title()}</span>
                  <span class="truncate text-right font-medium">{selected.title}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="shrink-0 text-muted-foreground">{m.projects_detail_language()}</span>
                  <span class="font-medium">{selected.language}</span>
                </div>
                {#if selected.creator}
                  <div class="flex justify-between gap-4">
                    <span class="shrink-0 text-muted-foreground">{m.projects_detail_creator()}</span>
                    <span class="truncate text-right font-medium">{selected.creator}</span>
                  </div>
                {/if}
                <div class="flex justify-between gap-4">
                  <span class="shrink-0 text-muted-foreground">{m.projects_detail_modified()}</span>
                  <span class="font-medium tabular-nums">{formatDate(selected.modified)}</span>
                </div>
              </div>

              <!-- 更多信息 -->
              <div class="rounded-lg border">
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="flex cursor-pointer items-center justify-between p-4" onclick={() => (showMore = !showMore)}>
                  <span class="text-sm font-medium">{m.projects_detail_more()}</span>
                  {#if showMore}
                    <ChevronUpIcon class="size-4 text-muted-foreground" />
                  {:else}
                    <ChevronDownIcon class="size-4 text-muted-foreground" />
                  {/if}
                </div>
                {#if showMore}
                  <div class="space-y-2 border-t p-4 text-sm">
                    <div class="flex justify-between gap-4">
                      <span class="shrink-0 text-muted-foreground">{m.projects_detail_identifier()}</span>
                      <span class="max-w-3/4 truncate text-right font-mono break-all" title={selected.identifier}
                        >{selected.identifier}</span
                      >
                    </div>
                    {#if selected.contributor}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_contributor()}</span>
                        <span class="truncate text-right font-medium">{selected.contributor}</span>
                      </div>
                    {/if}
                    {#if selected.publisher}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_publisher()}</span>
                        <span class="truncate text-right font-medium">{selected.publisher}</span>
                      </div>
                    {/if}
                    {#if selected.date}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_date()}</span>
                        <span class="font-medium">{selected.date}</span>
                      </div>
                    {/if}
                    {#if selected.subjects.length > 0}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_subjects()}</span>
                        <span class="truncate text-right font-medium">{selected.subjects.join(" / ")}</span>
                      </div>
                    {/if}
                    {#if selected.description.length > 0}
                      <div class="space-y-1">
                        <span class="text-muted-foreground">{m.projects_detail_description()}</span>
                        <div class="space-y-1">
                          {#each selected.description as line (line)}
                            <p class="text-sm break-words">{line}</p>
                          {/each}
                        </div>
                      </div>
                    {/if}
                    {#if selected.rights}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_rights()}</span>
                        <span class="truncate text-right font-medium">{selected.rights}</span>
                      </div>
                    {/if}
                    {#if selected.source}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_source()}</span>
                        <span class="truncate text-right font-medium">{selected.source}</span>
                      </div>
                    {/if}
                    {#if selected.relation}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_relation()}</span>
                        <span class="truncate text-right font-medium">{selected.relation}</span>
                      </div>
                    {/if}
                    {#if selected.coverage}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_coverage()}</span>
                        <span class="truncate text-right font-medium">{selected.coverage}</span>
                      </div>
                    {/if}
                    <div class="flex justify-between gap-4">
                      <span class="shrink-0 text-muted-foreground">{m.projects_detail_created()}</span>
                      <span class="font-medium tabular-nums">{formatDate(selected.created)}</span>
                    </div>
                    {#if selected.cover}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_cover()}</span>
                        <span class="max-w-48 truncate text-right text-xs break-all" title={selected.cover}
                          >{selected.cover}</span
                        >
                      </div>
                    {/if}
                    {#if selected.content}
                      <div class="flex justify-between gap-4">
                        <span class="shrink-0 text-muted-foreground">{m.projects_detail_content()}</span>
                        <span class="max-w-48 truncate text-right text-xs break-all" title={selected.content}
                          >{selected.content}</span
                        >
                      </div>
                    {/if}
                    {#if !selected.cover}
                      <p class="text-xs text-muted-foreground">{m.projects_detail_no_cover()}</p>
                    {/if}
                    {#if !selected.content}
                      <p class="text-xs text-muted-foreground">{m.projects_detail_no_content()}</p>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          </ScrollArea>

          <div class="shrink-0 border-t bg-card p-4">
            <ConfirmDialog
              title={m.projects_delete_confirm_title()}
              message={m.projects_delete_confirm_message({ name: selected.title })}
              variant="destructive"
              confirmLabel={m.projects_delete()}
              onConfirm={handleDelete}
            >
              {#snippet trigger({ props })}
                <Button variant="destructive" class="w-full" {...props}>
                  {m.projects_delete()}
                </Button>
              {/snippet}
            </ConfirmDialog>
          </div>
        </div>
      {/if}
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
