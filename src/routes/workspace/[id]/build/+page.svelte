<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { SvelteSet } from "svelte/reactivity";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import FileIcon from "@lucide/svelte/icons/file";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import HammerIcon from "@lucide/svelte/icons/hammer";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SearchIcon from "@lucide/svelte/icons/search";
  import WandIcon from "@lucide/svelte/icons/wand";
  import { Button } from "$components/ui/button";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import * as Resizable from "$components/ui/resizable";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Skeleton } from "$components/ui/skeleton";
  import { Textarea } from "$components/ui/textarea";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { getProject } from "$features/projects";
  import type { Project } from "$features/projects";
  import { getSplitContent } from "$features/split";
  import {
    getBuild,
    buildEpub,
    removeBuild,
    readBuildFile,
    writeBuildFile,
    getBuildPath,
    getFormat,
    formatBuildAll,
  } from "$features/build";
  import type { BuildFile, BuildResult, NumberFormat } from "$features/build";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";

  const id = $derived(page.params.id as string);
  const identifier = $derived(id ? (id.startsWith("urn:uuid:") ? id : `urn:uuid:${id}`) : "");

  let project = $state<Project | null>(null);
  let loadingProject = $state(true);
  let projectError = $state<string | null>(null);

  let splitExists = $state<boolean | null>(null);
  let loadingSplit = $state(true);

  let buildData = $state<BuildResult | null>(null);
  let loadingBuild = $state(true);
  let pending = $state(false);
  let saving = $state(false);
  let removing = $state(false);
  let formatting = $state(false);

  let search = $state("");
  let collapsedDirs = new SvelteSet<string>();
  let selectedPath = $state<string | null>(null);
  let originalContent = $state("");
  let draft = $state("");
  let loadingFile = $state(false);
  let isText = $state(true);

  // 构建格式状态
  let chapterTitleFormat = $state("第{order}章 {title}");
  let volumeTitleFormat = $state("第{order}卷 {title}");
  let numberFormat = $state<NumberFormat>("arabic");

  const hasBuild = $derived(!!buildData);
  const dirty = $derived(draft !== originalContent);

  const numberFormatOptions: Array<{ value: NumberFormat; label: () => string }> = [
    { value: "arabic", label: m.build_number_format_arabic },
    { value: "arabic_padded", label: m.build_number_format_padded },
    { value: "chinese_lower", label: m.build_number_format_chinese_lower },
    { value: "chinese_upper", label: m.build_number_format_chinese_upper },
  ];

  const numberFormatLabel = $derived(
    numberFormatOptions.find((o) => o.value === numberFormat)?.label() ?? m.build_number_format_arabic(),
  );

  function isTextFile(path: string): boolean {
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    return ["xhtml", "html", "xml", "opf", "css", "txt", "json"].includes(ext);
  }

  function filterTree(nodes: BuildFile[], query: string): BuildFile[] {
    const q = query.trim().toLowerCase();
    if (!q) return nodes;
    const out: BuildFile[] = [];
    for (const node of nodes) {
      if (node.isDir) {
        const filteredChildren = node.children ? filterTree(node.children, query) : [];
        const nameMatch = node.name.toLowerCase().includes(q);
        if (nameMatch) {
          out.push(node);
        } else if (filteredChildren.length > 0) {
          out.push({ ...node, children: filteredChildren });
        }
      } else {
        if (node.name.toLowerCase().includes(q)) {
          out.push(node);
        }
      }
    }
    return out;
  }

  const filteredFiles = $derived.by(() => {
    if (!buildData) return [] as BuildFile[];
    return filterTree(buildData.files, search);
  });

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

  async function loadSplitCheck() {
    if (!identifier) return;
    loadingSplit = true;
    try {
      const data = await getSplitContent(identifier);
      splitExists = !!data;
    } catch {
      splitExists = false;
    } finally {
      loadingSplit = false;
    }
  }

  async function loadBuild() {
    if (!identifier) return;
    loadingBuild = true;
    try {
      const data = await getBuild(identifier);
      if (data) {
        buildData = data;
      } else {
        buildData = null;
      }
    } catch {
      buildData = null;
    } finally {
      loadingBuild = false;
    }
  }

  async function loadFormat() {
    if (!identifier) return;
    try {
      const fmt = await getFormat(identifier);
      if (fmt) {
        if (fmt.chapterTitleFormat) chapterTitleFormat = fmt.chapterTitleFormat;
        if (fmt.volumeTitleFormat) volumeTitleFormat = fmt.volumeTitleFormat;
        if (fmt.numberFormat) numberFormat = fmt.numberFormat as NumberFormat;
      }
    } catch {
      // 忽略，回退默认值
    }
  }

  onMount(() => {
    void loadProject();
    void loadSplitCheck();
    void loadBuild();
    void loadFormat();
  });

  $effect(() => {
    if (identifier) {
      void loadProject();
      void loadSplitCheck();
      void loadBuild();
      void loadFormat();
    }
  });

  async function handleBuild() {
    if (!identifier) return;
    if (!splitExists) {
      toast.error(m.build_need_split());
      return;
    }
    if (!chapterTitleFormat.includes("{title}")) {
      toast.error(m.build_title_format_hint({ order: "{order}", title: "{title}" }));
      return;
    }
    if (!volumeTitleFormat.includes("{title}")) {
      toast.error(m.build_title_format_hint({ order: "{order}", title: "{title}" }));
      return;
    }
    pending = true;
    try {
      const res = await buildEpub(identifier, chapterTitleFormat, volumeTitleFormat, numberFormat);
      if (res) {
        buildData = res;
        selectedPath = null;
        originalContent = "";
        draft = "";
        collapsedDirs.clear();
        toast.success(m.build_success());
      } else {
        toast.error(m.build_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || m.build_failed());
    } finally {
      pending = false;
    }
  }

  async function handleRemoveBuild() {
    if (!identifier) return;
    removing = true;
    try {
      const ok = await removeBuild(identifier);
      if (ok) {
        buildData = null;
        selectedPath = null;
        originalContent = "";
        draft = "";
        collapsedDirs.clear();
        search = "";
        toast.success(m.build_remove_success());
      } else {
        toast.error(m.build_remove_failed());
      }
    } catch {
      toast.error(m.build_remove_failed());
    } finally {
      removing = false;
    }
  }

  async function handleFormatAll() {
    if (!identifier) return;
    formatting = true;
    try {
      const res = await formatBuildAll(identifier);
      if (res) {
        toast.success(m.build_format_success({ count: res.formatted }));
        const refreshed = await getBuild(identifier);
        if (refreshed) buildData = refreshed;
        if (selectedPath) {
          const ext = selectedPath.split(".").pop()?.toLowerCase() ?? "";
          if (["xhtml", "html", "xml", "opf", "css"].includes(ext)) {
            try {
              const content = await readBuildFile(identifier, selectedPath);
              if (content !== null) {
                // 若当前文件未被编辑，直接同步 draft；若有未保存修改则保留 draft 但更新 originalContent 使得 dirty 仍准确
                if (!dirty) draft = content;
                originalContent = content;
              }
            } catch {
              // 忽略刷新失败
            }
          }
        }
      } else {
        toast.error(m.build_format_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || m.build_format_failed());
    } finally {
      formatting = false;
    }
  }

  async function handleSelectFile(path: string, isDir: boolean) {
    if (isDir) {
      if (collapsedDirs.has(path)) collapsedDirs.delete(path);
      else collapsedDirs.add(path);
      return;
    }
    // file
    selectedPath = path;
    isText = isTextFile(path);
    if (!isText) {
      originalContent = "";
      draft = "";
      return;
    }
    loadingFile = true;
    try {
      const content = await readBuildFile(identifier, path);
      if (content !== null) {
        originalContent = content;
        draft = content;
      } else {
        toast.error(m.build_failed());
        originalContent = "";
        draft = "";
      }
    } catch {
      toast.error(m.build_failed());
      originalContent = "";
      draft = "";
    } finally {
      loadingFile = false;
    }
  }

  async function handleSave() {
    if (!selectedPath || !isText) return;
    saving = true;
    try {
      const ok = await writeBuildFile(identifier, selectedPath, draft);
      if (ok) {
        originalContent = draft;
        toast.success(m.build_save_success());
        // 刷新树（文件大小可能变，但结构不变，简单重载）
        const refreshed = await getBuild(identifier);
        if (refreshed) buildData = refreshed;
      } else {
        toast.error(m.build_save_failed());
      }
    } catch {
      toast.error(m.build_save_failed());
    } finally {
      saving = false;
    }
  }

  async function handleOpenFolder() {
    if (!buildData) return;
    try {
      // 直接揭示 epub 所在目录
      const p = await getBuildPath(identifier);
      if (p) {
        // getBuildPath returns Option<String> -> if null, use buildData.epubPath
        await revealItemInDir(p);
      } else {
        await revealItemInDir(buildData.epubPath);
      }
    } catch {
      toast.error(m.build_failed());
    }
  }

  function handleGoSplit() {
    void goto(resolve(`/workspace/${id}/split`));
  }
</script>

{#if loadingProject || loadingSplit || loadingBuild}
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
{:else if splitExists === false}
  <div class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center">
    <div class="space-y-2">
      <h3 class="text-sm font-semibold">{m.build_need_split()}</h3>
      <p class="text-xs text-muted-foreground">{m.build_need_split_hint()}</p>
    </div>
    <Button size="sm" onclick={handleGoSplit}>{m.build_go_split()}</Button>
  </div>
{:else if !hasBuild}
  <div class="flex flex-1 flex-col items-center justify-center overflow-auto p-6">
    <div class="flex w-full max-w-xl flex-col gap-5">
      <div class="space-y-1 text-center">
        <h3 class="text-sm font-semibold">{project?.title ?? ""}</h3>
        <p class="text-xs text-muted-foreground">{m.build_no_build_hint()}</p>
      </div>

      <div class="space-y-3 rounded-lg border p-4">
        <div class="space-y-1.5">
          <Label class="text-xs">{m.build_chapter_title_format_label()}</Label>
          <Input
            bind:value={chapterTitleFormat}
            placeholder={m.build_chapter_title_format_placeholder({ order: "{order}", title: "{title}" })}
            class="h-8 font-mono text-xs"
          />
        </div>
        <div class="space-y-1.5">
          <Label class="text-xs">{m.build_volume_title_format_label()}</Label>
          <Input
            bind:value={volumeTitleFormat}
            placeholder={m.build_volume_title_format_placeholder({ order: "{order}", title: "{title}" })}
            class="h-8 font-mono text-xs"
          />
        </div>
        <div class="space-y-1.5">
          <Label class="text-xs">{m.build_number_format_label()}</Label>
          <Select
            type="single"
            value={numberFormat}
            onValueChange={(v) => {
              if (v) numberFormat = v as NumberFormat;
            }}
          >
            <SelectTrigger class="h-8 text-xs">
              {numberFormatLabel}
            </SelectTrigger>
            <SelectContent>
              {#each numberFormatOptions as opt (opt.value)}
                <SelectItem value={opt.value}>{opt.label()}</SelectItem>
              {/each}
            </SelectContent>
          </Select>
        </div>
        <p class="text-xs text-muted-foreground">{m.build_title_format_hint({ order: "{order}", title: "{title}" })}</p>
      </div>

      <Button class="w-full gap-1.5" onclick={handleBuild} disabled={pending}>
        <HammerIcon class="size-4" />
        {pending ? m.build_building() : m.build_action()}
      </Button>
    </div>
  </div>
{:else if buildData}
  <Resizable.PaneGroup direction="horizontal" class="h-full min-h-0 w-full flex-1">
    <Resizable.Pane defaultSize={38} minSize={25} maxSize={50} class="flex min-h-0 flex-col overflow-hidden border-r">
      <div class="flex shrink-0 items-center gap-2 border-b p-2">
        <div class="relative flex-1">
          <SearchIcon class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input bind:value={search} placeholder={m.build_search_placeholder()} class="h-7 pl-7 text-xs" />
        </div>
        <Button
          variant="outline"
          size="sm"
          class="h-7 gap-1 text-xs"
          onclick={handleFormatAll}
          disabled={formatting || pending || removing}
        >
          <WandIcon class="size-3.5" />
          {formatting ? m.build_formatting() : m.build_format()}
        </Button>
        <ConfirmDialog
          title={m.build_rebuild_confirm_title()}
          message={m.build_rebuild_confirm_message()}
          variant="destructive"
          confirmLabel={m.build_rebuild()}
          onConfirm={handleRemoveBuild}
        >
          {#snippet trigger({ props })}
            <Button variant="outline" size="sm" class="h-7 gap-1 text-xs" {...props} disabled={removing || pending || formatting}>
              <RotateCcwIcon class="size-3.5" />
              {removing ? m.build_rebuilding() : m.build_rebuild()}
            </Button>
          {/snippet}
        </ConfirmDialog>
      </div>
      <ScrollArea class="h-full min-h-0 flex-1">
        <div class="p-2">
          {#if filteredFiles.length === 0}
            <p class="py-6 text-center text-xs text-muted-foreground">
              {search.trim() ? m.build_no_matched() : m.build_no_files()}
            </p>
          {:else}
            {#snippet tree(nodes: BuildFile[], depth: number)}
              <div class="space-y-0.5">
                {#each nodes as node (node.path)}
                  {#if node.isDir}
                    <div>
                      <button
                        class="flex w-full items-center gap-1 rounded-md px-1 py-1 text-left text-xs hover:bg-accent"
                        style="padding-left: {depth * 12 + 4}px"
                        onclick={() => handleSelectFile(node.path, true)}
                      >
                        <ChevronDownIcon
                          class="size-3.5 shrink-0 transition-transform {collapsedDirs.has(node.path) ? '-rotate-90' : ''}"
                        />
                        {#if collapsedDirs.has(node.path)}
                          <FolderIcon class="size-3.5 shrink-0 text-muted-foreground" />
                        {:else}
                          <FolderOpenIcon class="size-3.5 shrink-0 text-muted-foreground" />
                        {/if}
                        <span class="truncate">{node.name}</span>
                      </button>
                      {#if !collapsedDirs.has(node.path) && node.children}
                        {@render tree(node.children, depth + 1)}
                      {/if}
                    </div>
                  {:else}
                    <button
                      class="flex w-full items-center gap-1.5 rounded-md px-1 py-1 text-left text-xs hover:bg-accent {selectedPath === node.path
                        ? 'bg-accent font-medium'
                        : ''}"
                      style="padding-left: {depth * 12 + 20}px"
                      onclick={() => handleSelectFile(node.path, false)}
                    >
                      {#if isTextFile(node.path)}
                        <FileTextIcon class="size-3.5 shrink-0 text-muted-foreground" />
                      {:else}
                        <FileIcon class="size-3.5 shrink-0 text-muted-foreground" />
                      {/if}
                      <span class="flex-1 truncate">{node.name}</span>
                    </button>
                  {/if}
                {/each}
              </div>
            {/snippet}
            {@render tree(filteredFiles, 0)}
          {/if}
        </div>
      </ScrollArea>
      <div class="shrink-0 border-t p-2">
        <Button variant="outline" size="sm" class="w-full gap-1.5 text-xs" onclick={handleOpenFolder}>
          <FolderIcon class="size-3.5" />
          {m.build_open_folder()}
        </Button>
      </div>
    </Resizable.Pane>
    <Resizable.Handle withHandle />
    <Resizable.Pane defaultSize={62} minSize={40} class="flex min-h-0 flex-col overflow-hidden">
      {#if !selectedPath}
        <div class="flex flex-1 items-center justify-center p-6">
          <p class="text-sm text-muted-foreground">{m.build_select_hint()}</p>
        </div>
      {:else if !isText}
        <div class="flex flex-1 flex-col items-center justify-center gap-2 p-6">
          <FileIcon class="size-8 text-muted-foreground" />
          <p class="text-xs text-muted-foreground">{m.build_binary_hint()}</p>
          <p class="text-xs text-muted-foreground">{selectedPath}</p>
        </div>
      {:else if loadingFile}
        <div class="flex flex-1 items-center justify-center p-6">
          <Skeleton class="h-32 w-full" />
        </div>
      {:else}
        <div class="flex shrink-0 items-center justify-between gap-2 border-b p-3">
          <div class="min-w-0 flex-1">
            <h3 class="truncate font-mono text-xs font-medium">{selectedPath}</h3>
            <p class="text-xs text-muted-foreground">{m.build_edit_hint()}</p>
          </div>
          <Button size="sm" class="h-7 gap-1.5" onclick={handleSave} disabled={!dirty || saving}>
            <SaveIcon class="size-3.5" />
            {saving ? m.build_saving() : m.build_save()}
          </Button>
        </div>
        <div class="flex min-h-0 flex-1 flex-col p-3">
          <Textarea
            bind:value={draft}
            placeholder={selectedPath}
            class="min-h-0 flex-1 resize-none font-mono text-xs leading-5"
          />
        </div>
      {/if}
    </Resizable.Pane>
  </Resizable.PaneGroup>
{/if}
