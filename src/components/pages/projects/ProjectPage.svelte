<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$components/ui/button";
  import { Checkbox } from "$components/ui/checkbox";
  import { Input } from "$components/ui/input";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { mockProjects } from "$features/projects";
  import type { Project, ProjectSortKey } from "$features/projects";

  let projects = $state<Project[]>([...mockProjects]);
  let search = $state("");
  let sortKey = $state<ProjectSortKey>("modifiedDesc");
  let selectedId = $state<string | null>(mockProjects[0]?.identifier ?? null);
  let checked = new SvelteSet<string>();

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
      if (sortKey === "modifiedAsc") return a.modified - b.modified;
      return b.modified - a.modified;
    });
    return list;
  });

  const allChecked = $derived(filteredSorted.length > 0 && filteredSorted.every((p) => checked.has(p.identifier)));
  const indeterminate = $derived(!allChecked && filteredSorted.some((p) => checked.has(p.identifier)));

  function formatModified(ts: number): string {
    try {
      return new Date(ts).toLocaleString();
    } catch {
      return String(ts);
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
    toast.info(m.projects_new());
  }

  function handleDelete() {
    if (!selected) return;
    const id = selected.identifier;
    projects = projects.filter((p) => p.identifier !== id);
    checked.delete(id);
    toast.success(m.projects_delete_success());
    if (selectedId === id) {
      selectedId = projects[0]?.identifier ?? null;
    }
  }

  function handleBatchDelete() {
    if (checked.size === 0) return;
    const targets = new Set(checked);
    const before = projects.length;
    projects = projects.filter((p) => !targets.has(p.identifier));
    const deleted = before - projects.length;
    checked.clear();
    if (selectedId && targets.has(selectedId)) {
      selectedId = projects[0]?.identifier ?? null;
    }
    if (deleted > 0) toast.success(m.projects_batch_delete_success({ count: String(deleted) }));
    else toast.error(m.projects_batch_delete_failed());
  }
</script>

<div class="flex h-full w-full flex-col">
  <Resizable.PaneGroup direction="horizontal" class="h-full w-full">
    <!-- 左侧：项目列表 -->
    <Resizable.Pane defaultSize={50} minSize={40} maxSize={60} class="flex flex-col">
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

      <div class="flex-1 overflow-hidden">
        <ScrollArea class="h-full">
          <div class="flex flex-col gap-1 p-2">
            {#if filteredSorted.length === 0}
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
    <Resizable.Pane defaultSize={50} minSize={40} maxSize={60} class="flex flex-col bg-card">
      {#if !selected}
        <div class="flex h-full items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.projects_detail_empty()}</p>
        </div>
      {:else}
        <div class="flex h-full flex-col">
          <ScrollArea class="flex-1">
            <div class="space-y-4 p-6">
              <h2 class="truncate text-base font-semibold" title={selected.title}>{selected.title}</h2>
              <div class="space-y-3 rounded-lg border p-4 text-sm">
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.projects_detail_identifier()}</span>
                  <span class="max-w-48 truncate font-mono text-xs break-all" title={selected.identifier}
                    >{selected.identifier}</span
                  >
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.projects_detail_title()}</span>
                  <span class="font-medium">{selected.title}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.projects_detail_language()}</span>
                  <span class="font-medium">{selected.language}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.projects_detail_modified()}</span>
                  <span class="font-medium tabular-nums">{formatModified(selected.modified)}</span>
                </div>
              </div>
            </div>
          </ScrollArea>

          <div class="border-t p-4">
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
