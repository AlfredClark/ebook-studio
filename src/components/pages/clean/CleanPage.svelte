<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { open } from "@tauri-apps/plugin-dialog";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$components/ui/button";
  import { Checkbox } from "$components/ui/checkbox";
  import { Input } from "$components/ui/input";
  import * as Popover from "$components/ui/popover";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { listenFileDrop } from "$libs/drag-drop";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { copyTxt, deleteTxt, getTxtDetail, listTxts } from "$features/clean";
  import type { SortKey, TxtDetail, TxtInfo } from "$features/clean";

  let files = $state<TxtInfo[]>([]);
  let selected = $state<string | null>(null);
  let detail = $state<TxtDetail | null>(null);
  let detailLoading = $state(false);
  let search = $state("");
  let sortKey = $state<SortKey>("mtimeDesc");
  let loading = $state(false);
  let pending = $state(false);
  let dragging = $state(false);
  let addPopoverOpen = $state(false);
  let checked = new SvelteSet<string>();
  let unlistenFileDrop: (() => void) | undefined;
  let disposedFileDrop = false;

  const sortOptions: { value: SortKey; label: () => string }[] = [
    { value: "nameAsc", label: m.clean_sort_nameAsc },
    { value: "nameDesc", label: m.clean_sort_nameDesc },
    { value: "mtimeDesc", label: m.clean_sort_mtimeDesc },
    { value: "mtimeAsc", label: m.clean_sort_mtimeAsc },
    { value: "sizeDesc", label: m.clean_sort_sizeDesc },
    { value: "sizeAsc", label: m.clean_sort_sizeAsc },
  ];

  const sortLabel = $derived(sortOptions.find((o) => o.value === sortKey)?.label() ?? sortOptions[2].label());
  const checkedCount = $derived(checked.size);

  const filteredSorted = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = q ? files.filter((f) => f.name.toLowerCase().includes(q)) : files.slice();
    list.sort((a, b) => {
      switch (sortKey) {
        case "nameAsc":
          return a.name.localeCompare(b.name, undefined, { numeric: true });
        case "nameDesc":
          return b.name.localeCompare(a.name, undefined, { numeric: true });
        case "mtimeAsc":
          return a.mtime - b.mtime;
        case "mtimeDesc":
          return b.mtime - a.mtime;
        case "sizeAsc":
          return a.size - b.size;
        case "sizeDesc":
          return b.size - a.size;
        default:
          return 0;
      }
    });
    return list;
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function formatMtime(secs: number): string {
    if (!secs) return "-";
    try {
      return new Date(secs * 1000).toLocaleString();
    } catch {
      return String(secs);
    }
  }

  function toggleChecked(name: string, value: boolean) {
    if (value) checked.add(name);
    else checked.delete(name);
  }

  async function loadFiles() {
    loading = true;
    try {
      const data = await listTxts();
      files = data ?? [];
      // 若已选中但文件已不存在，清空详情
      if (selected && !files.some((f) => f.name === selected)) {
        selected = null;
        detail = null;
      }
      // 清理已不存在的勾选
      for (const name of Array.from(checked)) {
        if (!files.some((f) => f.name === name)) checked.delete(name);
      }
    } finally {
      loading = false;
    }
  }

  async function loadDetail(name: string) {
    detailLoading = true;
    detail = null;
    try {
      const d = await getTxtDetail(name);
      if (selected === name) {
        if (d) detail = d;
        else toast.error(m.clean_delete_failed());
      }
    } finally {
      if (selected === name) detailLoading = false;
    }
  }

  function handleSelect(name: string) {
    if (selected === name) return;
    selected = name;
    void loadDetail(name);
  }

  async function handleCopyPaths(paths: string[]) {
    if (pending || paths.length === 0) return;
    pending = true;
    let added = 0;
    try {
      for (const p of paths) {
        const res = await copyTxt(p);
        if (res === null) {
          toast.error(m.clean_copy_failed());
          continue;
        }
        added += res.length;
      }
      if (added > 0) {
        toast.success(m.clean_copy_success({ count: String(added) }));
        await loadFiles();
      } else {
        toast.error(m.clean_not_txt());
      }
    } finally {
      pending = false;
    }
  }

  async function handlePickFiles() {
    if (pending) return;
    addPopoverOpen = false;
    try {
      const picked = await open({
        multiple: true,
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!picked) return;
      const paths = Array.isArray(picked) ? picked : [picked];
      await handleCopyPaths(paths);
    } catch {
      toast.error(m.clean_copy_failed());
    }
  }

  async function handlePickFolder() {
    if (pending) return;
    addPopoverOpen = false;
    try {
      const picked = await open({ directory: true, multiple: false });
      if (!picked || Array.isArray(picked)) return;
      await handleCopyPaths([picked as string]);
    } catch {
      toast.error(m.clean_copy_failed());
    }
  }

  async function handleDelete() {
    if (!selected || pending) return;
    pending = true;
    try {
      const name = selected;
      const res = await deleteTxt(name);
      if (res) {
        toast.success(m.clean_delete_success());
        files = files.filter((f) => f.name !== name);
        checked.delete(name);
        selected = null;
        detail = null;
      } else {
        toast.error(m.clean_delete_failed());
      }
    } finally {
      pending = false;
    }
  }

  async function handleBatchDelete() {
    if (checked.size === 0 || pending) return;
    pending = true;
    const targets = Array.from(checked);
    const succeeded: string[] = [];
    let failed = 0;
    try {
      for (const name of targets) {
        const res = await deleteTxt(name);
        if (res) succeeded.push(name);
        else failed++;
      }
      if (succeeded.length > 0) {
        toast.success(m.clean_batch_delete_success({ count: String(succeeded.length) }));
        files = files.filter((f) => !succeeded.includes(f.name));
        for (const n of succeeded) checked.delete(n);
        if (selected && succeeded.includes(selected)) {
          selected = null;
          detail = null;
        }
        if (failed > 0) toast.error(m.clean_batch_delete_failed());
      } else {
        toast.error(m.clean_batch_delete_failed());
      }
    } finally {
      pending = false;
    }
  }

  onMount(() => {
    void loadFiles();
    void listenFileDrop((event) => {
      switch (event.payload.type) {
        case "enter":
        case "over":
          dragging = true;
          break;
        case "drop":
          dragging = false;
          void handleCopyPaths(event.payload.paths);
          break;
        case "leave":
          dragging = false;
          break;
      }
    })
      .then((fn) => {
        if (disposedFileDrop) fn();
        else unlistenFileDrop = fn;
      })
      .catch(() => {});
  });

  onDestroy(() => {
    disposedFileDrop = true;
    unlistenFileDrop?.();
  });
</script>

<div class="flex h-full w-full flex-col">
  <Resizable.PaneGroup direction="horizontal" class="h-full w-full">
    <!-- 左侧：文件列表 -->
    <Resizable.Pane defaultSize={60} minSize={30} class="flex flex-col">
      <div class="flex items-center gap-2 border-b p-3">
        <Popover.Root bind:open={addPopoverOpen}>
          <Popover.Trigger>
            {#snippet child({ props })}
              <Button {...props} variant="outline" size="sm" class="h-8 shrink-0 gap-1" disabled={pending}>
                <PlusIcon class="size-4" />
                {m.clean_add()}
              </Button>
            {/snippet}
          </Popover.Trigger>
          <Popover.Content class="w-36 p-1" align="start">
            <div class="flex flex-col gap-1">
              <Button variant="ghost" size="sm" class="justify-start" onclick={handlePickFiles} disabled={pending}>
                {m.clean_add_button()}
              </Button>
              <Button variant="ghost" size="sm" class="justify-start" onclick={handlePickFolder} disabled={pending}>
                {m.clean_add_folder_button()}
              </Button>
            </div>
          </Popover.Content>
        </Popover.Root>

        <Input bind:value={search} placeholder={m.clean_search_placeholder()} class="h-8 flex-1" />

        <Select
          type="single"
          value={sortKey}
          onValueChange={(v) => {
            if (v) sortKey = v as SortKey;
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
            title={m.clean_batch_delete_confirm_title()}
            message={m.clean_batch_delete_confirm_message({ count: String(checkedCount) })}
            variant="destructive"
            confirmLabel={m.clean_batch_delete_button({ count: String(checkedCount) })}
            onConfirm={handleBatchDelete}
          >
            {#snippet trigger({ props })}
              <Button variant="destructive" size="sm" class="h-8 shrink-0" disabled={pending} {...props}>
                {m.clean_batch_delete_button({ count: String(checkedCount) })}
              </Button>
            {/snippet}
          </ConfirmDialog>
        {/if}
      </div>

      <div class="relative flex-1 overflow-hidden">
        <ScrollArea class="h-full">
          <div class="flex flex-col gap-1 p-2">
            {#if loading}
              <p class="p-4 text-sm text-muted-foreground">...</p>
            {:else if filteredSorted.length === 0}
              <p class="p-4 text-center text-sm text-muted-foreground">
                {search.trim() ? m.clean_empty_filtered() : m.clean_empty()}
              </p>
            {:else}
              {#each filteredSorted as file (file.name)}
                <div
                  class="flex items-center gap-2 rounded-md px-2 py-1 transition-colors hover:bg-accent {selected === file.name
                    ? 'bg-accent text-accent-foreground'
                    : 'text-foreground'}"
                >
                  <Checkbox checked={checked.has(file.name)} onCheckedChange={(v) => toggleChecked(file.name, v === true)} />
                  <button
                    type="button"
                    onclick={() => handleSelect(file.name)}
                    class="flex min-w-0 flex-1 items-center justify-between gap-3 text-left text-sm"
                  >
                    <span class="min-w-0 flex-1 truncate font-medium">{file.name}</span>
                    <span class="shrink-0 text-xs text-muted-foreground">{formatSize(file.size)}</span>
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        </ScrollArea>

        <!-- 拖拽浮层 -->
        {#if dragging}
          <div
            class="pointer-events-none absolute inset-0 flex items-center justify-center border-2 border-dashed border-primary bg-accent/80 backdrop-blur-sm"
          >
            <p class="text-sm font-medium text-primary">{m.clean_drop_active()}</p>
          </div>
        {:else}
          <div
            class="pointer-events-none absolute inset-x-0 bottom-0 flex items-center justify-center p-2 text-xs text-muted-foreground opacity-60"
          >
            {m.clean_drop_hint()}
          </div>
        {/if}
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <!-- 右侧：详情 -->
    <Resizable.Pane defaultSize={40} minSize={30} maxSize={50} class="flex flex-col bg-card">
      {#if !selected}
        <div class="flex h-full items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.clean_detail_empty()}</p>
        </div>
      {:else if detailLoading}
        <div class="space-y-4 p-6">
          <div class="h-5 w-32 animate-pulse rounded bg-muted"></div>
          <div class="space-y-2">
            {#each [0, 1, 2, 3, 4] as item (item)}
              <div class="h-4 animate-pulse rounded bg-muted"></div>
            {/each}
          </div>
        </div>
      {:else if detail}
        <div class="flex h-full flex-col">
          <ScrollArea class="flex-1">
            <div class="space-y-4 p-6">
              <h2 class="truncate text-base font-semibold" title={detail.name}>{detail.name}</h2>
              <div class="space-y-3 rounded-lg border p-4 text-sm">
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.clean_detail_size()}</span>
                  <span class="font-medium">{formatSize(detail.size)}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.clean_detail_mtime()}</span>
                  <span class="font-medium">{formatMtime(detail.mtime)}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.clean_detail_lines()}</span>
                  <span class="font-medium tabular-nums">{detail.lines}</span>
                </div>
                <div class="flex justify-between gap-4">
                  <span class="text-muted-foreground">{m.clean_detail_chars()}</span>
                  <span class="font-medium tabular-nums">{detail.chars}</span>
                </div>
                <div class="space-y-1 pt-2">
                  <span class="text-muted-foreground">{m.clean_detail_path()}</span>
                  <p class="text-xs break-all text-muted-foreground" title={detail.path}>{detail.path}</p>
                </div>
              </div>
            </div>
          </ScrollArea>

          <div class="border-t p-4">
            <ConfirmDialog
              title={m.clean_delete_confirm_title()}
              message={m.clean_delete_confirm_message({ name: detail.name })}
              variant="destructive"
              confirmLabel={m.clean_delete_button()}
              onConfirm={handleDelete}
            >
              {#snippet trigger({ props })}
                <Button variant="destructive" class="w-full" disabled={pending} {...props}>
                  {m.clean_delete_button()}
                </Button>
              {/snippet}
            </ConfirmDialog>
          </div>
        </div>
      {:else}
        <div class="flex h-full items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.clean_detail_empty()}</p>
        </div>
      {/if}
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
