<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { SvelteSet } from "svelte/reactivity";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import FilterIcon from "@lucide/svelte/icons/filter";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import SaveIcon from "@lucide/svelte/icons/save";
  import { Badge } from "$components/ui/badge";
  import { Button } from "$components/ui/button";
  import { Checkbox } from "$components/ui/checkbox";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Separator } from "$components/ui/separator";
  import { Skeleton } from "$components/ui/skeleton";
  import { Textarea } from "$components/ui/textarea";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { getProject } from "$features/projects";
  import type { Project } from "$features/projects";
  import { getSplitContent, saveSplitChapter } from "$features/split";
  import type { SplitResult, SplitChapter, SplitVolume } from "$features/split";
  import { filterClean } from "$features/clean";
  import type { CleanFilters, CleanIssue, CleanResult } from "$features/clean";

  const id = $derived(page.params.id as string);
  const identifier = $derived(id ? (id.startsWith("urn:uuid:") ? id : `urn:uuid:${id}`) : "");

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let project = $state<Project | null>(null);
  let loadingProject = $state(true);
  let projectError = $state<string | null>(null);

  let splitData = $state<SplitResult | null>(null);
  let loadingSplit = $state(true);
  let saving = $state(false);

  let collapsed = new SvelteSet<number>();

  type Selected = { volumeOrder: number | null; chapterOrder: number } | null;
  let selected = $state<Selected>(null);
  let draft = $state("");

  let filters = $state<CleanFilters>({
    halfPunct: false,
    specialSymbol: false,
    unclosedPairs: false,
    repeatedPunct: false,
    abnormalWhitespace: false,
    ellipsisDash: false,
  });
  let customRegex = $state("");
  let filterPending = $state(false);
  let cleanResult = $state<CleanResult | null>(null);

  const hasSplit = $derived(!!splitData);
  const isPure = $derived(splitData?.type === "pure_chapters");
  const hasFiltered = $derived(!!cleanResult);

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

  // 左栏：基于 cleanResult 过滤，若无结果则显示全部
  const matchedSet = $derived.by(() => {
    if (!cleanResult) return null as Set<string> | null;
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const s = new Set<string>();
    for (const m of cleanResult.matched) {
      s.add(`${m.volumeOrder ?? "null"}:${m.chapterOrder}`);
    }
    return s;
  });

  const issuesBySelected = $derived.by(() => {
    if (!cleanResult || !selected) return [] as CleanIssue[];
    const sel = selected;
    return cleanResult.issues.filter(
      (i) => (i.volumeOrder ?? null) === (sel.volumeOrder ?? null) && i.chapterOrder === sel.chapterOrder,
    );
  });

  const filteredPureChapters = $derived.by(() => {
    if (!splitData || splitData.type !== "pure_chapters") return [] as SplitChapter[];
    const chs = splitData.chapters ?? [];
    if (!matchedSet) return chs;
    return chs.filter((c) => matchedSet.has(`null:${c.order}`));
  });

  const filteredVolumes = $derived.by(() => {
    if (!splitData || splitData.type === "pure_chapters") return [] as SplitVolume[];
    const vols = splitData.volumes ?? [];
    if (!matchedSet) return vols;
    const out: SplitVolume[] = [];
    for (const vol of vols) {
      const filteredChapters = vol.chapters.filter((c) => matchedSet.has(`${vol.order}:${c.order}`));
      if (filteredChapters.length > 0) {
        out.push({ ...vol, chapters: filteredChapters });
      }
    }
    return out;
  });

  const totalMatched = $derived(cleanResult?.matched.length ?? 0);

  const filterDefs: Array<{ key: keyof CleanFilters; label: () => string }> = [
    { key: "halfPunct", label: m.clean_filter_half_punct },
    { key: "specialSymbol", label: m.clean_filter_special_symbol },
    { key: "unclosedPairs", label: m.clean_filter_unclosed_pairs },
    { key: "repeatedPunct", label: m.clean_filter_repeated_punct },
    { key: "abnormalWhitespace", label: m.clean_filter_abnormal_whitespace },
    { key: "ellipsisDash", label: m.clean_filter_ellipsis_dash },
  ];

  function kindLabel(kind: string): string {
    const map: Record<string, () => string> = {
      half_punct: m.clean_issue_half_punct,
      special_symbol: m.clean_issue_special_symbol,
      unclosed_pairs: m.clean_issue_unclosed_pairs,
      repeated_punct: m.clean_issue_repeated_punct,
      abnormal_whitespace: m.clean_issue_abnormal_whitespace,
      ellipsis_dash: m.clean_issue_ellipsis_dash,
      custom_regex: m.clean_issue_custom_regex,
    };
    return (map[kind] ?? (() => kind))();
  }

  async function loadProject() {
    if (!identifier) return;
    loadingProject = true;
    projectError = null;
    try {
      const data = await getProject(identifier);
      if (data) {
        project = data;
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
        // 不自动选中，等待筛选或用户点击；但若已有筛选结果则选中首个匹配
        if (cleanResult && cleanResult.matched.length > 0) {
          const first = cleanResult.matched[0];
          selected = { volumeOrder: first.volumeOrder, chapterOrder: first.chapterOrder };
          // draft 延迟到 selectedChapter 派生后？直接设置
          const chap = (() => {
            if (data.type === "pure_chapters") {
              return data.chapters?.find((c) => c.order === first.chapterOrder) ?? null;
            } else {
              const vol = data.volumes?.find((v) => v.order === first.volumeOrder);
              return vol?.chapters.find((c) => c.order === first.chapterOrder) ?? null;
            }
          })();
          draft = chap ? chap.contents.join("\n") : "";
        }
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

  function handleSelect(volumeOrder: number | null, chapterOrder: number) {
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
        const updatedChap = (() => {
          if (res.type === "pure_chapters") {
            return res.chapters?.find((c) => c.order === selected!.chapterOrder) ?? null;
          } else {
            const vol = res.volumes?.find((v) => v.order === selected!.volumeOrder);
            return vol?.chapters.find((c) => c.order === selected!.chapterOrder) ?? null;
          }
        })();
        draft = updatedChap ? updatedChap.contents.join("\n") : draft;
        toast.success(m.clean_save_success());
        // 保存后自动重筛以刷新问题
        if (cleanResult) {
          await handleFilterInternal(false);
        }
      } else {
        toast.error(m.clean_save_failed());
      }
    } catch {
      toast.error(m.clean_save_failed());
    } finally {
      saving = false;
    }
  }

  async function handleFilterInternal(showToast = true) {
    if (!identifier) return;
    filterPending = true;
    try {
      const res = await filterClean(identifier, filters, customRegex);
      if (res) {
        cleanResult = res;
        if (res.matched.length === 0) {
          selected = null;
          draft = "";
          if (showToast) toast.success(m.clean_no_matched());
        } else {
          // 若当前选中仍在匹配则保留，否则切到首个
          const selKey = selected ? `${selected.volumeOrder ?? "null"}:${selected.chapterOrder}` : null;
          const stillMatched = selKey
            ? res.matched.some((mt) => `${mt.volumeOrder ?? "null"}:${mt.chapterOrder}` === selKey)
            : false;
          if (!stillMatched) {
            const first = res.matched[0];
            selected = { volumeOrder: first.volumeOrder, chapterOrder: first.chapterOrder };
            // draft 需等待 splitData 查询；此处直接取 splitData 当前值
            const chap = (() => {
              if (!splitData) return null;
              if (splitData.type === "pure_chapters") {
                return splitData.chapters?.find((c) => c.order === first.chapterOrder) ?? null;
              } else {
                const vol = splitData.volumes?.find((v) => v.order === first.volumeOrder);
                return vol?.chapters.find((c) => c.order === first.chapterOrder) ?? null;
              }
            })();
            draft = chap ? chap.contents.join("\n") : "";
          }
          if (showToast) toast.success(m.clean_matched_count({ count: String(res.matched.length) }));
        }
      } else {
        // 后端返回 null 时已在 filterClean 中 toast 记录，补一个通用提示
        toast.error(m.clean_filter_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes("正则")) {
        toast.error(m.clean_regex_invalid({ msg }));
      } else if (msg.includes("请选择")) {
        toast.error(m.clean_filter_need_selection());
      } else {
        toast.error(m.clean_filter_failed());
      }
    } finally {
      filterPending = false;
    }
  }

  async function handleFilter() {
    // 本地校验正则合法性（提前提示）
    if (customRegex.trim()) {
      try {
        new RegExp(customRegex);
      } catch (e) {
        toast.error(m.clean_regex_invalid({ msg: String(e) }));
        return;
      }
    }
    const anyChecked = Object.values(filters).some(Boolean);
    if (!anyChecked && !customRegex.trim()) {
      toast.error(m.clean_filter_need_selection());
      return;
    }
    await handleFilterInternal(true);
  }

  function handleReset() {
    filters = {
      halfPunct: false,
      specialSymbol: false,
      unclosedPairs: false,
      repeatedPunct: false,
      abnormalWhitespace: false,
      ellipsisDash: false,
    };
    customRegex = "";
    cleanResult = null;
    // 重置后左栏回到全部，选中清空
    selected = null;
    draft = "";
  }

  function toggleVolume(order: number) {
    if (collapsed.has(order)) collapsed.delete(order);
    else collapsed.add(order);
  }

  function handleGoSplit() {
    void goto(resolve(`/workspace/${id}/split`));
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
    <Button variant="outline" size="sm" onclick={() => void goto(resolve("/projects"))}>{m.workspace_back()}</Button>
  </div>
{:else if !hasSplit}
  <div class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center">
    <div class="space-y-2">
      <h3 class="text-sm font-semibold">{m.clean_need_split()}</h3>
      <p class="text-xs text-muted-foreground">{m.clean_need_split_hint()}</p>
    </div>
    <Button size="sm" onclick={handleGoSplit}>{m.clean_go_split()}</Button>
  </div>
{:else if splitData}
  <Resizable.PaneGroup direction="horizontal" class="h-full min-h-0 w-full flex-1">
    <!-- 左：章节列表（无搜索/按钮） -->
    <Resizable.Pane defaultSize={25} minSize={18} maxSize={35} class="flex min-h-0 flex-col overflow-hidden border-r">
      <div class="flex shrink-0 items-center justify-between border-b px-3 py-2">
        <h3 class="text-xs font-semibold">{m.clean_title()}</h3>
        {#if hasFiltered}
          <Badge variant="secondary" class="text-xs">{m.clean_matched_count({ count: String(totalMatched) })}</Badge>
        {:else}
          <span class="text-xs text-muted-foreground">{m.clean_no_filter_hint()}</span>
        {/if}
      </div>
      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="space-y-1 p-2">
          {#if isPure}
            {#if filteredPureChapters.length === 0}
              <p class="py-6 text-center text-xs text-muted-foreground">
                {#if hasFiltered}{m.clean_no_matched()}{:else}{m.clean_no_chapters()}{/if}
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
                {#if hasFiltered}{m.clean_no_matched_hint()}{:else}{m.clean_no_chapters()}{/if}
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
    <!-- 中：预览保存 + 本章问题 -->
    <Resizable.Pane defaultSize={45} minSize={30} class="flex min-h-0 flex-col overflow-hidden">
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
              {m.clean_contents_count({ count: String(selectedChapter.contents.length) })}
            </p>
          </div>
          <Button size="sm" class="h-7 gap-1.5" onclick={handleSave} disabled={!dirty || saving}>
            <SaveIcon class="size-3.5" />
            {saving ? m.clean_saving() : m.clean_save()}
          </Button>
        </div>
        <div class="flex min-h-0 flex-1 flex-col p-3">
          <Textarea
            bind:value={draft}
            placeholder={m.clean_contents_placeholder()}
            class="min-h-[180px] flex-1 resize-none font-mono text-xs leading-5"
          />
          <p class="pt-2 text-xs text-muted-foreground">{m.clean_edit_hint()}</p>
        </div>
        <Separator />
        <div class="flex min-h-0 flex-col">
          <div class="shrink-0 px-3 py-2">
            <h4 class="text-xs font-medium">
              {m.clean_issues_title({ count: String(issuesBySelected.length) })}
            </h4>
          </div>
          <ScrollArea class="h-48 min-h-0 flex-1 border-t">
            {#if issuesBySelected.length === 0}
              <p class="p-3 text-center text-xs text-muted-foreground">
                {hasFiltered ? m.clean_no_matched_hint() : m.clean_no_filter_hint()}
              </p>
            {:else}
              <div class="space-y-1 p-2">
                {#each issuesBySelected as issue (issue.lineIndex + issue.kind + issue.context)}
                  <div class="flex gap-2 rounded-md border p-2 text-xs">
                    <Badge variant="secondary" class="h-5 shrink-0 text-xs">{kindLabel(issue.kind)}</Badge>
                    <div class="min-w-0 flex-1 space-y-1">
                      <p class="truncate font-mono text-xs">{issue.context}</p>
                      <p class="text-muted-foreground">
                        {m.clean_line_no({ line: String(issue.lineIndex + 1) })}{#if issue.matched}
                          · {issue.matched}{/if}
                      </p>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </ScrollArea>
        </div>
      {:else}
        <div class="flex flex-1 items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.clean_select_hint()}</p>
        </div>
      {/if}
    </Resizable.Pane>
    <Resizable.Handle withHandle />
    <!-- 右：筛选面板 -->
    <Resizable.Pane defaultSize={30} minSize={22} maxSize={38} class="flex min-h-0 flex-col overflow-hidden border-l">
      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="space-y-4 p-4">
          <h3 class="text-sm font-semibold">{m.clean_filter_title()}</h3>
          <div class="space-y-3">
            {#each filterDefs as def (def.key)}
              <label class="flex items-center gap-2 text-sm">
                <Checkbox bind:checked={filters[def.key]} />
                <span>{def.label()}</span>
              </label>
            {/each}
          </div>
          <Separator />
          <div class="space-y-1.5">
            <Label class="text-xs">{m.clean_regex_label()}</Label>
            <Input bind:value={customRegex} placeholder={m.clean_regex_placeholder()} class="h-8 font-mono text-xs" />
          </div>
          <div class="flex gap-2">
            <Button size="sm" class="flex-1 gap-1.5" onclick={handleFilter} disabled={filterPending}>
              <FilterIcon class="size-3.5" />
              {filterPending ? m.clean_filtering() : m.clean_filter()}
            </Button>
            <Button variant="outline" size="sm" class="gap-1.5" onclick={handleReset}>
              <RotateCcwIcon class="size-3.5" />
              {m.clean_reset()}
            </Button>
          </div>
          <p class="text-xs text-muted-foreground">{m.clean_no_filter_hint()}</p>
        </div>
      </ScrollArea>
    </Resizable.Pane>
  </Resizable.PaneGroup>
{/if}
