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
  import { createProject } from "$features/projects";
  import { invokeCommand } from "$libs/ipc";
  import { CalendarDate, getLocalTimeZone } from "@internationalized/date";

  let title = $state("");
  let language = $state("zh-CN");
  let creator = $state("");
  let contributor = $state("");
  let publisher = $state("");
  let subjectsRaw = $state("");
  let description = $state("");
  let rights = $state("");
  let source = $state("");
  let relation = $state("");
  let coverage = $state("");
  let dateValue = $state<CalendarDate | undefined>(undefined);
  let datePopoverOpen = $state(false);

  let coverSrc = $state<string | null>(null);
  let coverPreview = $state<string | null>(null);
  let contentSrc = $state<string | null>(null);
  let contentSize = $state<string | null>(null);
  let contentChars = $state<number | null>(null);
  let contentFileName = $state<string | null>(null);

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
      await loadCoverPreview(path);
    } catch {
      toast.error(m.projects_create_failed());
    }
  }

  function handleRemoveCover() {
    coverSrc = null;
    coverPreview = null;
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
      await updateContentStats(path);
    } catch {
      toast.error(m.projects_create_failed());
    }
  }

  function handleRemoveContent() {
    contentSrc = null;
    contentFileName = null;
    contentSize = null;
    contentChars = null;
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
        await loadCoverPreview(p);
        continue;
      }
      if (dragOverContent && isTxt) {
        contentSrc = p;
        contentFileName = p.split("/").pop() ?? p.split("\\").pop() ?? p;
        await updateContentStats(p);
        continue;
      }
      if (isImage && !coverSrc) {
        coverSrc = p;
        coverPreview = null;
        await loadCoverPreview(p);
      } else if (isTxt && !contentSrc) {
        contentSrc = p;
        contentFileName = p.split("/").pop() ?? p.split("\\").pop() ?? p;
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
    } catch {
      toast.error(m.projects_create_failed());
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
</script>

<div class="flex h-full w-full flex-col overflow-hidden p-3">
  <div class="mx-auto flex w-full max-w-5xl flex-1 flex-col gap-3 overflow-hidden">
    <div class="flex shrink-0 items-center">
      <Button variant="ghost" size="sm" class="-ml-1 h-7 gap-1 px-2" onclick={handleBack}>
        <ArrowLeftIcon class="size-4" />
        {m.projects_create_back()}
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
            {#if coverPreview && coverSrc}
              <img src={coverPreview} alt="cover" class="h-full w-full object-cover" />
            {:else}
              <div class="flex flex-col items-center gap-1 p-3 text-center">
                <ImageIcon class="size-6 text-muted-foreground" />
                <p class="text-xs leading-tight text-muted-foreground">{m.projects_create_cover_drag_hint()}</p>
              </div>
            {/if}
          </div>
          <div class="flex gap-1.5">
            {#if coverSrc}
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
            {#if contentSrc}
              <FileTextIcon class="size-6 text-muted-foreground" />
              <p class="max-w-full truncate text-xs font-medium" title={contentSrc}>{contentFileName}</p>
              {#if contentSize && contentChars !== null}
                <p class="text-xs text-muted-foreground">
                  {m.projects_create_content_stats({ size: contentSize, chars: String(contentChars) })}
                </p>
              {/if}
            {:else}
              <FileTextIcon class="size-6 text-muted-foreground" />
              <p class="text-xs leading-tight text-muted-foreground">{m.projects_create_content_drag_hint()}</p>
            {/if}
          </div>
          <div class="flex gap-1.5">
            {#if contentSrc}
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
        {pending ? m.projects_create_submitting() : m.projects_create_submit()}
      </Button>
    </div>
  </div>
</div>
