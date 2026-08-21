<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import SaveIcon from "@lucide/svelte/icons/save";
  import ScissorsIcon from "@lucide/svelte/icons/scissors";
  import SearchIcon from "@lucide/svelte/icons/search";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import { Button } from "$components/ui/button";
  import { Input } from "$components/ui/input";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Skeleton } from "$components/ui/skeleton";
  import { Textarea } from "$components/ui/textarea";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { getProject, resolveProjectAsset } from "$features/projects";
  import type { Project } from "$features/projects";
  import { getSplitContent, splitContent, saveSplitChapter } from "$features/split";
  import type { SplitResult, SplitChapter, SplitVolume } from "$features/split";
  import { invokeCommand } from "$libs/ipc";

  const id = $derived(page.params.id as string);
  const identifier = $derived(id ? (id.startsWith("urn:uuid:") ? id : `urn:uuid:${id}`) : "");

  let project = $state<Project | null>(null);
  let loadingProject = $state(true);
  let projectError = $state<string | null>(null);
  let coverSrc = $state<string | null>(null);
  let hasContent = $derived(!!project?.content);

  let splitData = $state<SplitResult | null>(null);
  let loadingSplit = $state(true);
  let pending = $state(false);
  let saving = $state(false);

  import { SvelteSet } from "svelte/reactivity";

  let search = $state("");
  let collapsed = new SvelteSet<number>();

  type Selected = { volumeOrder: number | null; chapterOrder: number } | null;
  let selected = $state<Selected>(null);
  let draft = $state("");

  const hasSplit = $derived(!!splitData);
  const isPure = $derived(splitData?.type === "pure_chapters");

  // 派生：当前选中章节的内容（原始 join）
  const selectedChapter = $derived.by(() => {
    if (!splitData || !selected) return null as SplitChapter | null;
    const sel = selected;
    if (splitData.type === "pure_chapters") {
      return splitData.chapters?.find((c) => c.order === sel.chapterOrder) ?? null;
    } else {
      const vol = splitData.volumes?.find((v) => v.order === sel.volumeOrder);
      if (!vol) return null;
      return vol.chapters.find((c) => c.order === sel.chapterOrder) ?? null;
    }
  });

  const selectedContentsJoined = $derived(selectedChapter ? selectedChapter.contents.join("\n") : "");
  const dirty = $derived(draft !== selectedContentsJoined);

  // 过滤：按章节 title 搜索（大小写不敏感，trim）
  const filteredPureChapters = $derived.by(() => {
    if (!splitData || splitData.type !== "pure_chapters") return [] as SplitChapter[];
    const q = search.trim().toLowerCase();
    const chs = splitData.chapters ?? [];
    if (!q) return chs;
    return chs.filter((c) => c.title.toLowerCase().includes(q));
  });

  const filteredVolumes = $derived.by(() => {
    if (!splitData || splitData.type === "pure_chapters") return [] as SplitVolume[];
    const q = search.trim().toLowerCase();
    const vols = splitData.volumes ?? [];
    if (!q) return vols;
    return vols
      .map((v) => {
        const matchedChapters = v.chapters.filter((c) => c.title.toLowerCase().includes(q));
        // 若卷标题匹配则保留整卷，否则仅保留匹配章节
        const volTitleMatch = v.title.toLowerCase().includes(q);
        if (volTitleMatch) return v;
        if (matchedChapters.length > 0) return { ...v, chapters: matchedChapters };
        return null;
      })
      .filter((v): v is SplitVolume => v !== null);
  });

  const totalVolumes = $derived(splitData?.type !== "pure_chapters" ? (splitData?.volumes?.length ?? 0) : 0);
  const totalChapters = $derived.by(() => {
    if (!splitData) return 0;
    if (splitData.type === "pure_chapters") return splitData.chapters?.length ?? 0;
    return splitData.volumes?.reduce((acc, v) => acc + v.chapters.length, 0) ?? 0;
  });

  async function loadProject() {
    if (!identifier) return;
    loadingProject = true;
    projectError = null;
    try {
      const data = await getProject(identifier);
      if (data) {
        project = data;
        if (data.cover) {
          try {
            const abs = await resolveProjectAsset(data.identifier, data.cover);
            if (abs) {
              const dataUrl = await invokeCommand<string>("read_image_as_data_url", { path: abs });
              if (dataUrl) coverSrc = dataUrl;
            }
          } catch {
            coverSrc = null;
          }
        } else {
          coverSrc = null;
        }
      } else {
        projectError = m.workspace_load_failed();
      }
    } catch {
      projectError = m.workspace_load_failed();
    } finally {
      loadingProject = false;
    }
  }

  async function loadSplit() {
    if (!identifier) return;
    loadingSplit = true;
    try {
      const data = await getSplitContent(identifier);
      if (data) {
        splitData = data;
        // 默认选中首章
        autoSelectFirst();
      } else {
        splitData = null;
        selected = null;
        draft = "";
      }
    } catch {
      splitData = null;
    } finally {
      loadingSplit = false;
    }
  }

  function autoSelectFirst() {
    if (!splitData) return;
    if (splitData.type === "pure_chapters") {
      const first = splitData.chapters?.[0];
      if (first) {
        selected = { volumeOrder: null, chapterOrder: first.order };
        draft = first.contents.join("\n");
      }
    } else {
      const firstVol = splitData.volumes?.[0];
      const firstChap = firstVol?.chapters[0];
      if (firstVol && firstChap) {
        selected = { volumeOrder: firstVol.order, chapterOrder: firstChap.order };
        draft = firstChap.contents.join("\n");
      }
    }
  }

  onMount(() => {
    void loadProject();
    void loadSplit();
  });

  $effect(() => {
    if (identifier) {
      void loadProject();
      void loadSplit();
    }
  });

  async function handleSplit() {
    if (!identifier) return;
    if (!hasContent) {
      toast.error(m.split_need_content());
      return;
    }
    pending = true;
    try {
      const res = await splitContent(identifier);
      if (res) {
        splitData = res;
        collapsed.clear();
        autoSelectFirst();
        toast.success(m.split_success());
      } else {
        toast.error(m.split_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || m.split_failed());
    } finally {
      pending = false;
    }
  }

  async function handleResplit() {
    await handleSplit();
  }

  function handleSelect(volumeOrder: number | null, chapterOrder: number) {
    // 切换即丢弃未保存的 draft（需求）
    selected = { volumeOrder, chapterOrder };
    const chap = (() => {
      if (!splitData) return null;
      if (splitData.type === "pure_chapters") {
        return splitData.chapters?.find((c) => c.order === chapterOrder) ?? null;
      } else {
        const vol = splitData.volumes?.find((v) => v.order === volumeOrder);
        return vol?.chapters.find((c) => c.order === chapterOrder) ?? null;
      }
    })();
    draft = chap ? chap.contents.join("\n") : "";
  }

  async function handleSave() {
    if (!identifier || !selected || !splitData) return;
    saving = true;
    try {
      const contents = draft
        .split("\n")
        .map((s) => s.trim())
        .filter((s) => s.length > 0);
      const res = await saveSplitChapter(identifier, selected.volumeOrder, selected.chapterOrder, contents);
      if (res) {
        splitData = res;
        // 保存后同步 draft 为最新（避免 dirty 残留）
        // 重新定位选中章节的 contents（已更新）
        const updatedChap = (() => {
          if (res.type === "pure_chapters") {
            return res.chapters?.find((c) => c.order === selected!.chapterOrder) ?? null;
          } else {
            const vol = res.volumes?.find((v) => v.order === selected!.volumeOrder);
            return vol?.chapters.find((c) => c.order === selected!.chapterOrder) ?? null;
          }
        })();
        draft = updatedChap ? updatedChap.contents.join("\n") : draft;
        toast.success(m.split_save_success());
      } else {
        toast.error(m.split_save_failed());
      }
    } catch {
      toast.error(m.split_save_failed());
    } finally {
      saving = false;
    }
  }

  function toggleVolume(order: number) {
    if (collapsed.has(order)) collapsed.delete(order);
    else collapsed.add(order);
  }

  function handleBack() {
    void goto(resolve("/projects"));
  }
</script>

{#if loadingProject || loadingSplit}
  <div class="flex flex-1 items-center justify-center p-6">
    <div class="w-full max-w-xl space-y-3">
      <Skeleton class="h-6 w-32" />
      <Skeleton class="h-48 w-full" />
    </div>
  </div>
{:else if projectError}
  <div class="flex flex-1 flex-col items-center justify-center gap-3 p-6">
    <p class="text-sm text-destructive">{projectError}</p>
    <Button variant="outline" size="sm" onclick={handleBack}>{m.workspace_back()}</Button>
  </div>
{:else if !hasSplit}
  <!-- 初始：居中显示封面+信息+拆分按钮（与 inspect 初始态一致） -->
  <div class="flex flex-1 flex-col items-center justify-center overflow-auto p-6">
    <div class="flex w-full max-w-xl flex-col items-center gap-6">
      <div class="flex flex-col items-center gap-3">
        {#if coverSrc}
          <img src={coverSrc} alt="cover" class="aspect-3/4 w-48 rounded-lg border object-cover shadow-sm" />
        {:else}
          <div
            class="flex aspect-3/4 w-48 items-center justify-center rounded-lg border bg-muted text-xs text-muted-foreground"
          >
            {m.inspect_no_cover()}
          </div>
        {/if}
        <h2 class="text-center text-base font-semibold">{project?.title}</h2>
        {#if project?.creator}
          <p class="text-sm text-muted-foreground">{project.creator}</p>
        {/if}
      </div>

      <div class="flex w-full flex-col gap-3">
        <Button class="w-full gap-1.5" onclick={handleSplit} disabled={pending || !hasContent}>
          <ScissorsIcon class="size-4" />
          {pending ? m.split_splitting() : m.split_action()}
        </Button>
        {#if !hasContent}
          <p class="text-center text-xs text-muted-foreground">{m.split_need_content()}</p>
        {:else}
          <p class="text-center text-xs text-muted-foreground">{m.split_no_split_hint()}</p>
        {/if}
      </div>
    </div>
  </div>
{:else if splitData}
  <!-- 结果态：左右分栏（与 inspect 结果态一致） -->
  <Resizable.PaneGroup direction="horizontal" class="h-full min-h-0 w-full flex-1">
    <Resizable.Pane defaultSize={38} minSize={30} maxSize={45} class="flex min-h-0 flex-col overflow-hidden border-r">
      <div class="flex shrink-0 flex-col gap-2 border-b p-3">
        <div class="flex items-center gap-2">
          <Button variant="ghost" size="icon" class="size-7 shrink-0" onclick={handleBack} aria-label={m.workspace_back()}>
            <ArrowLeftIcon class="size-4" />
          </Button>
          <div class="relative flex-1">
            <SearchIcon class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input bind:value={search} placeholder={m.split_search_placeholder()} class="h-7 pl-7 text-xs" />
          </div>
          <ConfirmDialog
            title={m.split_resplit_confirm_title()}
            message={m.split_resplit_confirm_message()}
            variant="destructive"
            confirmLabel={m.split_resplit()}
            onConfirm={handleResplit}
          >
            {#snippet trigger({ props })}
              <Button variant="outline" size="sm" class="h-7 gap-1 text-xs" {...props} disabled={pending}>
                <RefreshCwIcon class="size-3.5" />
                {pending ? m.split_resplitting() : m.split_resplit()}
              </Button>
            {/snippet}
          </ConfirmDialog>
        </div>
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{m.split_stats_volumes({ count: String(totalVolumes) })}</span>
          <span>·</span>
          <span>{m.split_stats_chapters({ count: String(totalChapters) })}</span>
        </div>
      </div>

      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="space-y-1 p-2">
          {#if isPure}
            {#if filteredPureChapters.length === 0}
              <p class="py-6 text-center text-xs text-muted-foreground">
                {search.trim() ? m.split_empty_search() : m.split_no_chapters()}
              </p>
            {:else}
              {#each filteredPureChapters as ch (ch.order)}
                <button
                  class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs transition-colors hover:bg-accent {selected?.chapterOrder ===
                    ch.order && selected?.volumeOrder === null
                    ? 'bg-accent font-medium'
                    : ''}"
                  onclick={() => handleSelect(null, ch.order)}
                >
                  <span class="shrink-0 text-muted-foreground">#{ch.order}</span>
                  <span class="flex-1 truncate">{ch.title || m.split_untitled()}</span>
                </button>
              {/each}
            {/if}
          {:else}
            {#if filteredVolumes.length === 0}
              <p class="py-6 text-center text-xs text-muted-foreground">
                {search.trim() ? m.split_empty_search() : m.split_no_chapters()}
              </p>
            {:else}
              {#each filteredVolumes as vol (vol.order)}
                <div class="space-y-1">
                  <button
                    class="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-xs font-medium hover:bg-accent"
                    onclick={() => toggleVolume(vol.order)}
                  >
                    <ChevronDownIcon
                      class="size-3.5 shrink-0 transition-transform {collapsed.has(vol.order) ? '-rotate-90' : ''}"
                    />
                    <span class="flex-1 truncate">{m.split_volume_label({ order: String(vol.order), title: vol.title })}</span>
                    <span class="shrink-0 text-muted-foreground"
                      >{m.split_chapter_count({ count: String(vol.chapters.length) })}</span
                    >
                  </button>
                  {#if !collapsed.has(vol.order)}
                    <div class="ml-2 space-y-0.5 border-l pl-2">
                      {#each vol.chapters as ch (ch.order)}
                        <button
                          class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs transition-colors hover:bg-accent {selected?.volumeOrder ===
                            vol.order && selected?.chapterOrder === ch.order
                            ? 'bg-accent font-medium'
                            : ''}"
                          onclick={() => handleSelect(vol.order, ch.order)}
                        >
                          <span class="shrink-0 text-muted-foreground">#{ch.order}</span>
                          <span class="flex-1 truncate">{ch.title || m.split_untitled()}</span>
                        </button>
                      {/each}
                      {#if vol.chapters.length === 0}
                        <p class="px-2 py-1 text-xs text-muted-foreground">{m.split_no_chapters()}</p>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            {/if}
          {/if}
        </div>
      </ScrollArea>
    </Resizable.Pane>
    <Resizable.Handle withHandle />
    <Resizable.Pane defaultSize={62} minSize={40} class="flex min-h-0 flex-col overflow-hidden">
      {#if selectedChapter}
        <div class="flex shrink-0 items-center justify-between gap-2 border-b p-3">
          <div class="min-w-0 flex-1">
            <h3 class="truncate text-sm font-semibold">
              {#if !isPure && selected?.volumeOrder != null}
                {m.split_chapter_title_with_volume({
                  volumeOrder: String(selected.volumeOrder),
                  chapterOrder: String(selected.chapterOrder),
                  title: selectedChapter.title || m.split_untitled(),
                })}
              {:else}
                {m.split_chapter_title({
                  order: String(selectedChapter.order),
                  title: selectedChapter.title || m.split_untitled(),
                })}
              {/if}
            </h3>
            <p class="text-xs text-muted-foreground">
              {m.split_contents_count({ count: String(selectedChapter.contents.length) })}
            </p>
          </div>
          <Button size="sm" class="h-7 gap-1.5" onclick={handleSave} disabled={!dirty || saving}>
            <SaveIcon class="size-3.5" />
            {saving ? m.split_saving() : m.split_save()}
          </Button>
        </div>
        <div class="flex min-h-0 flex-1 flex-col p-3">
          <Textarea
            bind:value={draft}
            placeholder={m.split_contents_placeholder()}
            class="min-h-0 flex-1 resize-none font-mono text-xs leading-5"
          />
          <p class="pt-2 text-xs text-muted-foreground">{m.split_edit_hint()}</p>
        </div>
      {:else}
        <div class="flex flex-1 items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.split_select_hint()}</p>
        </div>
      {/if}
    </Resizable.Pane>
  </Resizable.PaneGroup>
{/if}
