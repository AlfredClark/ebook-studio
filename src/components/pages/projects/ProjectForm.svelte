<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { open } from "@tauri-apps/plugin-dialog";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import ImageIcon from "@lucide/svelte/icons/image";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import XIcon from "@lucide/svelte/icons/x";
  import { Calendar } from "$components/ui/calendar";
  import { Button } from "$components/ui/button";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import * as Popover from "$components/ui/popover";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Textarea } from "$components/ui/textarea";
  import { listenFileDrop } from "$libs/drag-drop";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { createProject, resolveProjectAsset, updateProject } from "$features/projects";
  import type { Project } from "$features/projects";
  import { invokeCommand } from "$libs/ipc";
  import { CalendarDate, getLocalTimeZone } from "@internationalized/date";

  let {
    mode = "create",
    initial,
  }: {
    mode?: "create" | "edit";
    initial?: Project | null;
  } = $props();

  const isEdit = $derived(mode === "edit");

  // 初始化：edit 时从 initial 回填，create 时空
  let title = $derived(initial?.title ?? "");
  let language = $derived(initial?.language ?? "zh-CN");
  let creator = $derived(initial?.creator ?? "");
  let contributor = $derived(initial?.contributor ?? "");
  let publisher = $derived(initial?.publisher ?? "");
  let subjectsRaw = $derived(initial?.subjects?.join(" / ") ?? "");
  let description = $derived(initial?.description?.join("\n") ?? "");
  let rights = $derived(initial?.rights ?? "");
  let source = $derived(initial?.source ?? "");
  let relation = $derived(initial?.relation ?? "");
  let coverage = $derived(initial?.coverage ?? "");
  let dateValue = $derived<CalendarDate | undefined>(parseDate(initial?.date ?? null));
  let datePopoverOpen = $state(false);

  let coverSrc = $state<string | null>(null);
  let coverPreview = $state<string | null>(null);
  let existingCoverPreview = $state<string | null>(null);
  let removeCover = $state(false);

  let contentSrc = $state<string | null>(null);
  let contentSize = $state<string | null>(null);
  let contentChars = $state<number | null>(null);
  let contentFileName = $state<string | null>(null);
  let existingContentName = $state<string | null>(null);
  let existingContentSize = $state<string | null>(null);
  let existingContentChars = $state<number | null>(null);
  let removeContent = $state(false);

  let pending = $state(false);
  let titleError = $state<string | null>(null);
  let languageError = $state<string | null>(null);

  let dragOverCover = $state(false);
  let dragOverContent = $state(false);
  let unlistenFileDrop: (() => void) | undefined;
  let disposedFileDrop = false;

  const languageOptions = [
    { value: "zh-CN", label: "简体中文" },
    { value: "en", label: "English" },
  ] as const;

  const languageLabel = $derived(languageOptions.find((o) => o.value === language)?.label ?? language);

  function parseDate(raw: string | null | undefined): CalendarDate | undefined {
    if (!raw) return undefined;
    // 期望 YYYY-MM-DD
    const m = raw.match(/^(\d{4})-(\d{2})-(\d{2})/);
    if (!m) return undefined;
    const y = Number(m[1]);
    const mo = Number(m[2]);
    const d = Number(m[3]);
    if (!y || !mo || !d) return undefined;
    try {
      return new CalendarDate(y, mo, d);
    } catch {
      return undefined;
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  async function updateContentStats(path: string) {
    try {
      const stats = await invokeCommand<{ size: number; chars: number }>("get_file_stats", { path });
      if (stats) {
        contentSize = formatSize(stats.size);
        contentChars = stats.chars;
      } else {
        contentSize = null;
        contentChars = null;
      }
    } catch {
      contentSize = null;
      contentChars = null;
    }
  }

  async function loadCoverPreview(path: string) {
    try {
      const dataUrl = await invokeCommand<string>("read_image_as_data_url", { path });
      if (dataUrl && coverSrc === path) coverPreview = dataUrl;
    } catch {
      coverPreview = null;
    }
  }

  // 加载编辑模式下的现有资产预览
  $effect(() => {
    if (!isEdit || !initial) {
      existingCoverPreview = null;
      existingContentName = null;
      existingContentSize = null;
      existingContentChars = null;
      return;
    }
    // 封面
    if (initial.cover && !coverSrc && !removeCover) {
      const id = initial.identifier;
      const rel = initial.cover;
      void (async () => {
        try {
          const abs = await resolveProjectAsset(id, rel);
          if (!abs) return;
          const dataUrl = await invokeCommand<string>("read_image_as_data_url", { path: abs });
          if (dataUrl) existingCoverPreview = dataUrl;
        } catch {
          existingCoverPreview = null;
        }
      })();
    } else if (!initial.cover) {
      existingCoverPreview = null;
    }
    // 正文
    if (initial.content && !contentSrc && !removeContent) {
      const id = initial.identifier;
      const rel = initial.content;
      existingContentName = rel.split("/").pop() ?? rel;
      void (async () => {
        try {
          const abs = await resolveProjectAsset(id, rel);
          if (!abs) return;
          const stats = await invokeCommand<{ size: number; chars: number }>("get_file_stats", { path: abs });
          if (stats) {
            existingContentSize = formatSize(stats.size);
            existingContentChars = stats.chars;
          }
        } catch {
          existingContentSize = null;
          existingContentChars = null;
        }
      })();
    } else if (!initial.content) {
      existingContentName = null;
      existingContentSize = null;
      existingContentChars = null;
    }
  });

  async function handlePickCover() {
    try {
      const picked = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }],
      });
      if (!picked || Array.isArray(picked)) return;
      const path = picked as string;
      coverSrc = path;
      coverPreview = null;
      removeCover = false;
      await loadCoverPreview(path);
    } catch {
      toast.error(isEdit ? m.projects_edit_failed() : m.projects_create_failed());
    }
  }

  function handleRemoveCover() {
    if (coverSrc) {
      coverSrc = null;
      coverPreview = null;
    } else if (isEdit && (existingCoverPreview || initial?.cover)) {
      removeCover = true;
      existingCoverPreview = null;
    } else {
      coverSrc = null;
      coverPreview = null;
    }
  }

  function handleRestoreCover() {
    removeCover = false;
    // 重新触发加载
    if (initial?.cover) {
      const id = initial.identifier;
      const rel = initial.cover;
      void (async () => {
        try {
          const abs = await resolveProjectAsset(id, rel);
          if (!abs) return;
          const dataUrl = await invokeCommand<string>("read_image_as_data_url", { path: abs });
          if (dataUrl) existingCoverPreview = dataUrl;
        } catch {
          existingCoverPreview = null;
        }
      })();
    }
  }

  async function handlePickContent() {
    try {
      const picked = await open({
        multiple: false,
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!picked || Array.isArray(picked)) return;
      const path = picked as string;
      contentSrc = path;
      contentFileName = path.split("/").pop() ?? path.split("\\").pop() ?? path;
      removeContent = false;
      await updateContentStats(path);
    } catch {
      toast.error(isEdit ? m.projects_edit_failed() : m.projects_create_failed());
    }
  }

  function handleRemoveContent() {
    if (contentSrc) {
      contentSrc = null;
      contentFileName = null;
      contentSize = null;
      contentChars = null;
    } else if (isEdit && (existingContentName || initial?.content)) {
      removeContent = true;
      existingContentName = null;
      existingContentSize = null;
      existingContentChars = null;
    } else {
      contentSrc = null;
      contentFileName = null;
      contentSize = null;
      contentChars = null;
    }
  }

  function handleRestoreContent() {
    removeContent = false;
    if (initial?.content) {
      const rel = initial.content;
      existingContentName = rel.split("/").pop() ?? rel;
      const id = initial.identifier;
      void (async () => {
        try {
          const abs = await resolveProjectAsset(id, rel);
          if (!abs) return;
          const stats = await invokeCommand<{ size: number; chars: number }>("get_file_stats", { path: abs });
          if (stats) {
            existingContentSize = formatSize(stats.size);
            existingContentChars = stats.chars;
          }
        } catch {
          existingContentSize = null;
          existingContentChars = null;
        }
      })();
    }
  }

  function handleCoverDragEnter(e: DragEvent) {
    e.preventDefault();
    dragOverCover = true;
  }
  function handleCoverDragOver(e: DragEvent) {
    e.preventDefault();
    dragOverCover = true;
  }
  function handleCoverDragLeave(e: DragEvent) {
    e.preventDefault();
    dragOverCover = false;
  }
  async function handleCoverDrop(e: DragEvent) {
    e.preventDefault();
    dragOverCover = false;
  }

  function handleContentDragEnter(e: DragEvent) {
    e.preventDefault();
    dragOverContent = true;
  }
  function handleContentDragOver(e: DragEvent) {
    e.preventDefault();
    dragOverContent = true;
  }
  function handleContentDragLeave(e: DragEvent) {
    e.preventDefault();
    dragOverContent = false;
  }
  async function handleContentDrop(e: DragEvent) {
    e.preventDefault();
    dragOverContent = false;
  }

  async function handleFileDropPaths(paths: string[]) {
    for (const p of paths) {
      const lower = p.toLowerCase();
      const isImage = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".bmp"].some((ext) => lower.endsWith(ext));
      const isTxt = lower.endsWith(".txt");
      if (dragOverCover && isImage) {
        coverSrc = p;
        coverPreview = null;
        removeCover = false;
        await loadCoverPreview(p);
        continue;
      }
      if (dragOverContent && isTxt) {
        contentSrc = p;
        contentFileName = p.split("/").pop() ?? p.split("\\").pop() ?? p;
        removeContent = false;
        await updateContentStats(p);
        continue;
      }
      if (isImage && !coverSrc && !existingCoverPreview) {
        coverSrc = p;
        coverPreview = null;
        removeCover = false;
        await loadCoverPreview(p);
      } else if (isTxt && !contentSrc && !existingContentName) {
        contentSrc = p;
        contentFileName = p.split("/").pop() ?? p.split("\\").pop() ?? p;
        removeContent = false;
        await updateContentStats(p);
      }
    }
  }

  onMount(() => {
    void listenFileDrop((event) => {
      const payload = event.payload;
      if (payload.type === "drop") {
        void handleFileDropPaths(payload.paths);
        dragOverCover = false;
        dragOverContent = false;
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

  function validate(): boolean {
    let ok = true;
    titleError = null;
    languageError = null;
    if (!title.trim()) {
      titleError = m.projects_create_field_title_required();
      ok = false;
    }
    if (!language.trim()) {
      languageError = m.projects_create_field_language_required();
      ok = false;
    } else if (!["en", "zh-CN"].includes(language)) {
      languageError = m.projects_create_field_language_required();
      ok = false;
    }
    return ok;
  }

  async function handleSubmit() {
    if (pending) return;
    if (!validate()) return;
    pending = true;
    try {
      const dateStr = dateValue ? dateValue.toString() : undefined;
      if (isEdit) {
        if (!initial) {
          toast.error(m.projects_edit_failed());
          return;
        }
        const result = await updateProject({
          identifier: initial.identifier,
          title: title.trim(),
          language: language.trim(),
          creator: creator.trim() || undefined,
          contributor: contributor.trim() || undefined,
          publisher: publisher.trim() || undefined,
          date: dateStr,
          subjects: subjectsRaw.trim() || undefined,
          description: description.trim() || undefined,
          rights: rights.trim() || undefined,
          source: source.trim() || undefined,
          relation: relation.trim() || undefined,
          coverage: coverage.trim() || undefined,
          coverSrc: coverSrc,
          contentSrc: contentSrc,
          removeCover: removeCover || undefined,
          removeContent: removeContent || undefined,
        });
        if (result) {
          toast.success(m.projects_edit_success());
          await goto(resolve("/projects"));
        } else {
          toast.error(m.projects_edit_failed());
        }
      } else {
        const result = await createProject({
          title: title.trim(),
          language: language.trim(),
          creator: creator.trim() || undefined,
          contributor: contributor.trim() || undefined,
          publisher: publisher.trim() || undefined,
          date: dateStr,
          subjects: subjectsRaw.trim() || undefined,
          description: description.trim() || undefined,
          rights: rights.trim() || undefined,
          source: source.trim() || undefined,
          relation: relation.trim() || undefined,
          coverage: coverage.trim() || undefined,
          coverSrc: coverSrc,
          contentSrc: contentSrc,
        });
        if (result) {
          toast.success(m.projects_create_success());
          await goto(resolve("/projects"));
        } else {
          toast.error(m.projects_create_failed());
        }
      }
    } catch {
      toast.error(isEdit ? m.projects_edit_failed() : m.projects_create_failed());
    } finally {
      pending = false;
    }
  }

  function handleBack() {
    void goto(resolve("/projects"));
  }

  function handleCancel() {
    void goto(resolve("/projects"));
  }

  const dateDisplay = $derived.by(() => {
    if (!dateValue) return "";
    try {
      const d = dateValue.toDate(getLocalTimeZone());
      return d.toLocaleDateString();
    } catch {
      return dateValue.toString();
    }
  });

  const coverDisplayUrl = $derived.by(() => {
    if (removeCover) return null;
    if (coverPreview && coverSrc) return coverPreview;
    if (existingCoverPreview) return existingCoverPreview;
    return null;
  });

  const hasExistingCover = $derived(!!initial?.cover && !removeCover && !coverSrc);
  const showCoverRemove = $derived(!!coverSrc || hasExistingCover);
  const showCoverRestore = $derived(removeCover);

  const hasContent = $derived(!!contentSrc || (!removeContent && !!existingContentName));
  const contentDisplayName = $derived.by(() => {
    if (contentSrc) return contentFileName;
    if (!removeContent && existingContentName) return existingContentName;
    return null;
  });
  const contentDisplayStats = $derived.by(() => {
    if (contentSrc && contentSize && contentChars !== null) return { size: contentSize, chars: contentChars };
    if (!removeContent && existingContentSize && existingContentChars !== null)
      return { size: existingContentSize, chars: existingContentChars };
    return null;
  });
</script>

<div class="flex h-full w-full flex-col overflow-hidden p-3">
  <div class="mx-auto flex w-full max-w-5xl flex-1 flex-col gap-3 overflow-hidden">
    <div class="flex shrink-0 items-center">
      <Button variant="ghost" size="sm" class="-ml-1 h-7 gap-1 px-2" onclick={handleBack}>
        <ArrowLeftIcon class="size-4" />
        {isEdit ? m.projects_edit_title() : m.projects_create_back()}
      </Button>
    </div>
    <div class="grid flex-1 grid-cols-1 gap-3 overflow-hidden lg:grid-cols-[200px_1fr]">
      <!-- 左侧：文件选择（缩小） -->
      <div class="flex flex-col gap-3 overflow-hidden">
        <!-- 封面 -->
        <div class="space-y-1.5">
          <Label class="text-xs"
            >{m.projects_create_cover_label()}
            <span class="text-xs font-normal text-muted-foreground">{m.projects_create_cover_hint()}</span></Label
          >
          <div
            role="button"
            tabindex="0"
            class="relative mx-auto flex aspect-[3/4] w-full max-w-[210px] cursor-pointer items-center justify-center overflow-hidden rounded-lg border-2 border-dashed bg-muted/20 transition-colors {dragOverCover
              ? 'border-primary bg-accent'
              : 'border-muted-foreground/25 hover:border-muted-foreground/50'}"
            onclick={handlePickCover}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                void handlePickCover();
              }
            }}
            ondragenter={handleCoverDragEnter}
            ondragover={handleCoverDragOver}
            ondragleave={handleCoverDragLeave}
            ondrop={handleCoverDrop}
          >
            {#if coverDisplayUrl}
              <img src={coverDisplayUrl} alt="cover" class="h-full w-full object-cover" />
            {:else}
              <div class="flex flex-col items-center gap-1 p-3 text-center">
                <ImageIcon class="size-6 text-muted-foreground" />
                <p class="text-xs leading-tight text-muted-foreground">{m.projects_create_cover_drag_hint()}</p>
              </div>
            {/if}
          </div>
          <div class="flex gap-1.5">
            {#if showCoverRestore}
              <Button variant="outline" size="sm" class="h-7 flex-1 text-xs" onclick={handleRestoreCover}
                >{m.projects_create_cover_pick()}</Button
              >
              <span class="flex h-7 items-center px-1 text-xs text-muted-foreground">{m.projects_detail_no_cover()}</span>
            {:else if showCoverRemove}
              <Button variant="outline" size="sm" class="h-7 flex-1 text-xs" onclick={handlePickCover}
                >{m.projects_create_cover_change()}</Button
              >
              <Button variant="ghost" size="sm" class="h-7 px-2" onclick={handleRemoveCover}><XIcon class="size-3.5" /></Button>
            {:else}
              <Button variant="outline" size="sm" class="h-7 w-full text-xs" onclick={handlePickCover}
                >{m.projects_create_cover_pick()}</Button
              >
            {/if}
          </div>
        </div>

        <!-- 正文 -->
        <div class="space-y-1.5">
          <Label class="text-xs">{m.projects_create_content_label()}</Label>
          <div
            role="button"
            tabindex="0"
            class="relative flex min-h-24 w-full cursor-pointer flex-col items-center justify-center gap-1.5 rounded-lg border-2 border-dashed p-3 transition-colors {dragOverContent
              ? 'border-primary bg-accent'
              : 'border-muted-foreground/25 hover:border-muted-foreground/50'}"
            onclick={handlePickContent}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                void handlePickContent();
              }
            }}
            ondragenter={handleContentDragEnter}
            ondragover={handleContentDragOver}
            ondragleave={handleContentDragLeave}
            ondrop={handleContentDrop}
          >
            {#if hasContent && contentDisplayName}
              <FileTextIcon class="size-6 text-muted-foreground" />
              <p class="max-w-full truncate text-xs font-medium" title={contentDisplayName}>{contentDisplayName}</p>
              {#if contentDisplayStats}
                <p class="text-xs text-muted-foreground">
                  {m.projects_create_content_stats({
                    size: contentDisplayStats.size,
                    chars: String(contentDisplayStats.chars),
                  })}
                </p>
              {/if}
            {:else}
              <FileTextIcon class="size-6 text-muted-foreground" />
              <p class="text-xs leading-tight text-muted-foreground">{m.projects_create_content_drag_hint()}</p>
            {/if}
          </div>
          <div class="flex gap-1.5">
            {#if removeContent}
              <Button variant="outline" size="sm" class="h-7 flex-1 text-xs" onclick={handleRestoreContent}
                >{m.projects_create_content_pick()}</Button
              >
              <span class="flex h-7 items-center px-1 text-xs text-muted-foreground">{m.projects_detail_no_content()}</span>
            {:else if hasContent}
              <Button variant="outline" size="sm" class="h-7 flex-1 text-xs" onclick={handlePickContent}
                >{m.projects_create_content_change()}</Button
              >
              <Button variant="ghost" size="sm" class="h-7 px-2" onclick={handleRemoveContent}
                ><XIcon class="size-3.5" /></Button
              >
            {:else}
              <Button variant="outline" size="sm" class="h-7 w-full text-xs" onclick={handlePickContent}
                >{m.projects_create_content_pick()}</Button
              >
            {/if}
          </div>
        </div>
      </div>

      <!-- 右侧：表单（紧凑） -->
      <div class="flex flex-col gap-2.5 overflow-hidden">
        <div class="grid grid-cols-1 gap-2.5">
          <div class="space-y-1">
            <Label for="title" class="text-xs">{m.projects_create_field_title()} <span class="text-destructive">*</span></Label>
            <Input
              id="title"
              bind:value={title}
              placeholder={m.projects_create_field_title_placeholder()}
              class="h-8 text-sm"
            />
            {#if titleError}
              <p class="text-xs text-destructive">{titleError}</p>
            {/if}
          </div>

          <div class="space-y-1">
            <Label class="text-xs">{m.projects_create_field_language()} <span class="text-destructive">*</span></Label>
            <Select
              type="single"
              value={language}
              onValueChange={(v) => {
                if (v) language = v;
              }}
            >
              <SelectTrigger class="h-8 w-full text-sm">
                {languageLabel}
              </SelectTrigger>
              <SelectContent>
                {#each languageOptions as opt (opt.value)}
                  <SelectItem value={opt.value}>{opt.label}</SelectItem>
                {/each}
              </SelectContent>
            </Select>
            {#if languageError}
              <p class="text-xs text-destructive">{languageError}</p>
            {/if}
          </div>

          <div class="grid grid-cols-1 gap-2.5 md:grid-cols-2">
            <div class="space-y-1">
              <Label for="creator" class="text-xs">{m.projects_create_field_creator()}</Label>
              <Input
                id="creator"
                bind:value={creator}
                placeholder={m.projects_create_field_creator_placeholder()}
                class="h-8 text-sm"
              />
            </div>
            <div class="space-y-1">
              <Label for="contributor" class="text-xs">{m.projects_create_field_contributor()}</Label>
              <Input
                id="contributor"
                bind:value={contributor}
                placeholder={m.projects_create_field_contributor_placeholder()}
                class="h-8 text-sm"
              />
            </div>
          </div>

          <div class="grid grid-cols-1 gap-2.5 md:grid-cols-2">
            <div class="space-y-1">
              <Label for="publisher" class="text-xs">{m.projects_create_field_publisher()}</Label>
              <Input
                id="publisher"
                bind:value={publisher}
                placeholder={m.projects_create_field_publisher_placeholder()}
                class="h-8 text-sm"
              />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">{m.projects_create_field_date()}</Label>
              <Popover.Root bind:open={datePopoverOpen}>
                <Popover.Trigger>
                  {#snippet child({ props })}
                    <Button {...props} variant="outline" class="h-8 w-full justify-start gap-2 text-sm font-normal">
                      <CalendarIcon class="size-3.5" />
                      {#if dateDisplay}
                        <span>{dateDisplay}</span>
                      {:else}
                        <span class="text-muted-foreground">{m.projects_create_field_date_placeholder()}</span>
                      {/if}
                    </Button>
                  {/snippet}
                </Popover.Trigger>
                <Popover.Content class="w-auto p-0" align="start">
                  <Calendar
                    type="single"
                    bind:value={dateValue}
                    captionLayout="dropdown"
                    onValueChange={(v) => {
                      if (v) {
                        dateValue = v as CalendarDate;
                        datePopoverOpen = false;
                      }
                    }}
                  />
                </Popover.Content>
              </Popover.Root>
            </div>
          </div>

          <div class="space-y-1">
            <Label for="subjects" class="text-xs"
              >{m.projects_create_field_subjects()}
              <span class="text-xs font-normal text-muted-foreground">{m.projects_create_field_subjects_hint()}</span></Label
            >
            <Input
              id="subjects"
              bind:value={subjectsRaw}
              placeholder={m.projects_create_field_subjects_placeholder()}
              class="h-8 text-sm"
            />
          </div>

          <div class="space-y-1">
            <Label for="description" class="text-xs">{m.projects_create_field_description()}</Label>
            <Textarea
              id="description"
              bind:value={description}
              placeholder={m.projects_create_field_description_placeholder()}
              class="min-h-14 resize-none text-sm"
              rows={2}
            />
          </div>

          <div class="grid grid-cols-1 gap-2.5 md:grid-cols-2">
            <div class="space-y-1">
              <Label for="rights" class="text-xs">{m.projects_create_field_rights()}</Label>
              <Input
                id="rights"
                bind:value={rights}
                placeholder={m.projects_create_field_rights_placeholder()}
                class="h-8 text-sm"
              />
            </div>
            <div class="space-y-1">
              <Label for="source" class="text-xs">{m.projects_create_field_source()}</Label>
              <Input
                id="source"
                bind:value={source}
                placeholder={m.projects_create_field_source_placeholder()}
                class="h-8 text-sm"
              />
            </div>
          </div>

          <div class="grid grid-cols-1 gap-2.5 md:grid-cols-2">
            <div class="space-y-1">
              <Label for="relation" class="text-xs">{m.projects_create_field_relation()}</Label>
              <Input
                id="relation"
                bind:value={relation}
                placeholder={m.projects_create_field_relation_placeholder()}
                class="h-8 text-sm"
              />
            </div>
            <div class="space-y-1">
              <Label for="coverage" class="text-xs">{m.projects_create_field_coverage()}</Label>
              <Input
                id="coverage"
                bind:value={coverage}
                placeholder={m.projects_create_field_coverage_placeholder()}
                class="h-8 text-sm"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部按钮（页面内部居中，宽一些） -->
    <div class="flex shrink-0 items-center justify-center gap-4 pt-1">
      <Button variant="outline" class="w-44" onclick={handleCancel} disabled={pending}>{m.projects_create_cancel()}</Button>
      <Button class="w-44" onclick={handleSubmit} disabled={pending}>
        {#if pending}
          {isEdit ? m.projects_edit_submitting() : m.projects_create_submitting()}
        {:else}
          {isEdit ? m.projects_edit_submit() : m.projects_create_submit()}
        {/if}
      </Button>
    </div>
  </div>
</div>
