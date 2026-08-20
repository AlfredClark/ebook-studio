<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import SearchIcon from "@lucide/svelte/icons/search";
  import WrenchIcon from "@lucide/svelte/icons/wrench";
  import { Badge } from "$components/ui/badge";
  import { Button } from "$components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle } from "$components/ui/card";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Skeleton } from "$components/ui/skeleton";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { getProject, resolveProjectAsset } from "$features/projects";
  import type { Project } from "$features/projects";
  import { getContentPath, inspectContent, reorderChapters } from "$features/inspect";
  import type { InspectResult, InspectStructure } from "$features/inspect";
  import { invokeCommand } from "$libs/ipc";

  const id = $derived(page.params.id as string);
  const identifier = $derived(id ? (id.startsWith("urn:uuid:") ? id : `urn:uuid:${id}`) : "");

  let project = $state<Project | null>(null);
  let loadingProject = $state(true);
  let projectError = $state<string | null>(null);
  let coverSrc = $state<string | null>(null);
  let hasContent = $derived(!!project?.content);

  let structure = $state<InspectStructure>("auto");
  let volumeRegex = $state("");
  let chapterRegex = $state("");
  let pending = $state(false);
  let reorderPending = $state(false);
  let result = $state<InspectResult | null>(null);
  let absPath = $state<string | null>(null);

  const hasReorder = $derived(
    !!result?.issues.some((i) => i.kind === "missing" || i.kind === "duplicate" || i.kind === "out_of_order"),
  );

  const DEFAULT_VOL_RE = String.raw`^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*卷\s*[:：]?\s*(.*)$`;
  const DEFAULT_CHAP_RE = String.raw`^\s*第\s*([0-9零一二三四五六七八九十百千万两〇]+)\s*章\s*(.*)$`;

  const hasResult = $derived(!!result);

  const structureOptions: { value: InspectStructure; label: () => string }[] = [
    { value: "auto", label: m.inspect_structure_auto },
    { value: "volume_chapters", label: m.inspect_structure_volume_chapters },
    { value: "pure_chapters", label: m.inspect_structure_pure_chapters },
    { value: "volume_intro", label: m.inspect_structure_volume_intro },
  ];

  const structureLabel = $derived(structureOptions.find((o) => o.value === structure)?.label() ?? m.inspect_structure_auto());
  const structureDisplay = (val: string) => {
    const map: Record<string, () => string> = {
      volume_chapters: m.inspect_structure_volume_chapters,
      pure_chapters: m.inspect_structure_pure_chapters,
      volume_intro: m.inspect_structure_volume_intro,
      auto: m.inspect_structure_auto,
    };
    return (map[val] ?? (() => val))();
  };

  const kindLabel = (kind: string) => {
    const map: Record<string, () => string> = {
      missing: m.inspect_issue_missing,
      duplicate: m.inspect_issue_duplicate,
      out_of_order: m.inspect_issue_out_of_order,
      invalid_number: m.inspect_issue_invalid_number,
      empty_volume: m.inspect_issue_empty_volume,
      no_chapters: m.inspect_issue_no_chapters,
    };
    return (map[kind] ?? (() => kind))();
  };

  async function loadProject() {
    if (!identifier) return;
    loadingProject = true;
    projectError = null;
    try {
      const data = await getProject(identifier);
      if (data) {
        project = data;
        // cover
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
        // absPath for file ops
        try {
          absPath = await getContentPath(data.identifier);
        } catch {
          absPath = result?.absPath ?? null;
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

  onMount(() => {
    void loadProject();
  });

  $effect(() => {
    if (identifier) void loadProject();
  });

  async function handleInspect() {
    if (!identifier) return;
    if (!hasContent) {
      toast.error(m.inspect_need_content());
      return;
    }
    pending = true;
    try {
      const res = await inspectContent(identifier, structure, volumeRegex, chapterRegex);
      if (res) {
        result = res;
        if (res.absPath) absPath = res.absPath;
        else {
          absPath = await getContentPath(identifier);
        }
      } else {
        // 空时区分正则错误与通用失败
        if (volumeRegex.trim() || chapterRegex.trim()) {
          // 尝试本地校验正则合法性，优先提示
          try {
            if (volumeRegex.trim()) new RegExp(volumeRegex);
            if (chapterRegex.trim()) new RegExp(chapterRegex);
          } catch (e) {
            toast.error(m.inspect_regex_invalid({ msg: String(e) }));
            return;
          }
        }
        toast.error(m.inspect_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes("正则")) toast.error(m.inspect_regex_invalid({ msg }));
      else toast.error(m.inspect_failed());
    } finally {
      pending = false;
    }
  }

  async function handleReorder() {
    if (!identifier || !hasContent) return;
    reorderPending = true;
    try {
      const res = await reorderChapters(identifier, structure, volumeRegex, chapterRegex);
      if (res) {
        result = res;
        if (res.absPath) absPath = res.absPath;
        toast.success(m.inspect_reorder_success());
      } else {
        toast.error(m.inspect_reorder_failed());
      }
    } catch {
      toast.error(m.inspect_reorder_failed());
    } finally {
      reorderPending = false;
    }
  }

  async function handleOpenFolder() {
    if (!absPath) {
      const p = await getContentPath(identifier);
      if (!p) return;
      absPath = p;
    }
    if (absPath) {
      try {
        await revealItemInDir(absPath);
      } catch {
        toast.error(m.inspect_failed());
      }
    }
  }

  function formatDate(v: string | number) {
    try {
      const d = new Date(v);
      if (isNaN(d.getTime())) return String(v);
      return d.toLocaleString();
    } catch {
      return String(v);
    }
  }
</script>

{#if loadingProject}
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
{:else if !hasResult}
  <!-- 初始：居中显示封面+信息+结构选择+检查按钮 -->
  <div class="flex flex-1 flex-col items-center justify-start overflow-auto p-6">
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

      <Card class="w-full">
        <CardHeader class="pb-3">
          <CardTitle class="text-sm">{m.inspect_title()}</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3 text-sm">
          <div class="flex justify-between">
            <span class="text-muted-foreground">{m.projects_detail_title()}</span>
            <span class="font-medium">{project?.title}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">{m.projects_detail_language()}</span>
            <span class="font-medium">{project?.language}</span>
          </div>
          {#if project?.creator}
            <div class="flex justify-between">
              <span class="text-muted-foreground">{m.projects_detail_creator()}</span>
              <span class="font-medium">{project.creator}</span>
            </div>
          {/if}
          <div class="flex justify-between">
            <span class="text-muted-foreground">{m.projects_detail_modified()}</span>
            <span class="font-medium tabular-nums">{project ? formatDate(project.modified) : ""}</span>
          </div>
          {#if project?.content}
            <div class="flex justify-between">
              <span class="text-muted-foreground">{m.projects_detail_content()}</span>
              <span class="max-w-48 truncate text-xs break-all" title={project.content}>{project.content}</span>
            </div>
          {:else}
            <p class="text-xs text-amber-600 dark:text-amber-400">{m.inspect_no_content()}</p>
          {/if}
        </CardContent>
      </Card>

      <div class="flex w-full flex-col gap-3">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium">{m.inspect_structure_label()}</span>
          <Select
            type="single"
            value={structure}
            onValueChange={(v) => {
              if (v) structure = v as InspectStructure;
            }}
          >
            <SelectTrigger class="h-8 flex-1">
              {structureLabel}
            </SelectTrigger>
            <SelectContent>
              {#each structureOptions as opt (opt.value)}
                <SelectItem value={opt.value}>{opt.label()}</SelectItem>
              {/each}
            </SelectContent>
          </Select>
        </div>
        <div class="space-y-1">
          <Label class="text-xs">{m.inspect_volume_regex_label()}</Label>
          <Input bind:value={volumeRegex} placeholder={DEFAULT_VOL_RE} class="h-8 font-mono text-xs" />
        </div>
        <div class="space-y-1">
          <Label class="text-xs">{m.inspect_chapter_regex_label()}</Label>
          <Input bind:value={chapterRegex} placeholder={DEFAULT_CHAP_RE} class="h-8 font-mono text-xs" />
        </div>
        <Button class="w-full gap-1.5" onclick={handleInspect} disabled={pending || !hasContent}>
          <SearchIcon class="size-4" />
          {pending ? m.inspect_checking() : m.inspect_check()}
        </Button>
        {#if !hasContent}
          <p class="text-center text-xs text-muted-foreground">{m.inspect_need_content()}</p>
        {/if}
      </div>
    </div>
  </div>
{:else if result}
  <!-- 结果态：左右分栏 -->
  <Resizable.PaneGroup direction="horizontal" class="h-full min-h-0 w-full flex-1">
    <Resizable.Pane defaultSize={38} minSize={30} maxSize={45} class="flex min-h-0 flex-col overflow-hidden border-r">
      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="flex flex-col items-center gap-4 p-6">
          {#if coverSrc}
            <img src={coverSrc} alt="cover" class="aspect-3/4 w-40 rounded-lg border object-cover shadow-sm" />
          {:else}
            <div
              class="flex aspect-3/4 w-40 items-center justify-center rounded-lg border bg-muted text-xs text-muted-foreground"
            >
              {m.inspect_no_cover()}
            </div>
          {/if}
          <div class="text-center">
            <h2 class="text-sm font-semibold">{project?.title}</h2>
            {#if project?.creator}
              <p class="text-xs text-muted-foreground">{project.creator}</p>
            {/if}
          </div>
          <Card class="w-full">
            <CardContent class="space-y-2 p-4 text-xs">
              <div class="flex justify-between">
                <span class="text-muted-foreground">{m.projects_detail_title()}</span>
                <span class="max-w-32 truncate text-right font-medium">{project?.title}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">{m.projects_detail_language()}</span>
                <span class="font-medium">{project?.language}</span>
              </div>
              {#if project?.creator}
                <div class="flex justify-between">
                  <span class="text-muted-foreground">{m.projects_detail_creator()}</span>
                  <span class="font-medium">{project.creator}</span>
                </div>
              {/if}
              <div class="flex justify-between">
                <span class="text-muted-foreground">{m.projects_detail_modified()}</span>
                <span class="font-medium tabular-nums">{project ? formatDate(project.modified) : ""}</span>
              </div>
            </CardContent>
          </Card>
          <div class="flex w-full flex-col gap-2">
            <div class="flex items-center gap-2">
              <Select
                type="single"
                value={structure}
                onValueChange={(v) => {
                  if (v) structure = v as InspectStructure;
                }}
              >
                <SelectTrigger class="h-7 flex-1 text-xs">
                  {structureLabel}
                </SelectTrigger>
                <SelectContent>
                  {#each structureOptions as opt (opt.value)}
                    <SelectItem value={opt.value}>{opt.label()}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
              <Button size="sm" class="h-7 gap-1" onclick={handleInspect} disabled={pending}>
                <SearchIcon class="size-3.5" />
                {pending ? m.inspect_checking() : m.inspect_rescan()}
              </Button>
            </div>
            <div class="space-y-1">
              <Label class="text-xs">{m.inspect_volume_regex_label()}</Label>
              <Input bind:value={volumeRegex} placeholder={DEFAULT_VOL_RE} class="h-7 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">{m.inspect_chapter_regex_label()}</Label>
              <Input bind:value={chapterRegex} placeholder={DEFAULT_CHAP_RE} class="h-7 font-mono text-xs" />
            </div>
          </div>
        </div>
      </ScrollArea>
    </Resizable.Pane>
    <Resizable.Handle withHandle />
    <Resizable.Pane defaultSize={62} minSize={40} class="flex min-h-0 flex-col overflow-hidden">
      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="space-y-4 p-6">
          <div class="flex flex-wrap items-center gap-2">
            <Badge variant="secondary">{m.inspect_detected({ structure: structureDisplay(result.detectedStructure) })}</Badge>
            {#if result.requestedStructure !== result.detectedStructure}
              <Badge variant="outline">{m.inspect_requested({ structure: structureDisplay(result.requestedStructure) })}</Badge>
            {/if}
          </div>

          <div class="grid grid-cols-2 gap-2 text-sm md:grid-cols-4">
            <Card>
              <CardContent class="p-3 text-center">
                <div class="text-xs text-muted-foreground">
                  {m.inspect_stats_volumes({ count: String(result.stats.totalVolumes) })}
                </div>
                <div class="text-lg font-semibold">{result.stats.totalVolumes}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="p-3 text-center">
                <div class="text-xs text-muted-foreground">
                  {m.inspect_stats_chapters({ count: String(result.stats.totalChapters) })}
                </div>
                <div class="text-lg font-semibold">{result.stats.totalChapters}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="p-3 text-center">
                <div class="text-xs text-muted-foreground">
                  {m.inspect_stats_issues({ count: String(result.issues.length) })}
                </div>
                <div class="text-lg font-semibold">{result.issues.length}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="p-3 text-center">
                <div class="text-xs text-muted-foreground">
                  {m.inspect_stats_lines({ count: String(result.stats.totalLines) })}
                </div>
                <div class="text-lg font-semibold">{result.stats.totalLines}</div>
              </CardContent>
            </Card>
          </div>

          {#if result.issues.length === 0}
            <div class="rounded-lg border bg-card p-6 text-center text-sm text-muted-foreground">
              {m.inspect_no_issues()}
            </div>
          {:else}
            <div class="space-y-2">
              <h3 class="text-sm font-medium">{m.inspect_issues_title({ count: String(result.issues.length) })}</h3>
              <div class="space-y-2">
                {#each result.issues as issue (issue.message + String(issue.lineNo))}
                  <div class="flex gap-3 rounded-md border p-3 text-xs">
                    <Badge
                      variant={issue.kind === "missing" || issue.kind === "no_chapters" ? "destructive" : "secondary"}
                      class="h-5 shrink-0"
                    >
                      {kindLabel(issue.kind)}
                    </Badge>
                    <div class="flex-1 space-y-1">
                      <p class="text-sm wrap-break-word">{issue.message}</p>
                      {#if issue.lineNo}
                        <p class="text-muted-foreground">
                          行 {issue.lineNo}{#if issue.context}
                            · {issue.context}{/if}
                        </p>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>

              <div class="flex gap-2 pt-2">
                {#if hasReorder}
                  <ConfirmDialog
                    title={m.inspect_reorder_confirm_title()}
                    message={m.inspect_reorder_confirm_message()}
                    variant="destructive"
                    confirmLabel={m.inspect_reorder()}
                    onConfirm={handleReorder}
                  >
                    {#snippet trigger({ props })}
                      <Button variant="outline" class="flex-1 gap-1.5" {...props} disabled={reorderPending}>
                        <WrenchIcon class="size-4" />
                        {reorderPending ? m.inspect_reordering() : m.inspect_reorder()}
                      </Button>
                    {/snippet}
                  </ConfirmDialog>
                {/if}
                <Button variant="outline" class={hasReorder ? "flex-1 gap-1.5" : "w-full gap-1.5"} onclick={handleOpenFolder}>
                  <FolderIcon class="size-4" />
                  {m.inspect_open_folder()}
                </Button>
              </div>
            </div>
          {/if}

          {#if result.volumes.length > 0}
            <div class="space-y-1 pt-2">
              <h4 class="text-xs font-medium text-muted-foreground">卷</h4>
              <div class="space-y-1 text-xs">
                {#each result.volumes as v (v.lineNo)}
                  <div class="flex justify-between rounded border px-2 py-1">
                    <span>第{v.rawNum ?? v.num}卷 {v.title}（行 {v.lineNo}）</span>
                    <span class="text-muted-foreground">{v.chapterCount} 章{v.hasIntro ? " · 有简介" : ""}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </ScrollArea>
    </Resizable.Pane>
  </Resizable.PaneGroup>
{/if}
