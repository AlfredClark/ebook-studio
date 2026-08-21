<script lang="ts">
  import { onMount, tick } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CheckCircleIcon from "@lucide/svelte/icons/check-circle";
  import FileIcon from "@lucide/svelte/icons/file";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import ImageIcon from "@lucide/svelte/icons/image";
  import PackageIcon from "@lucide/svelte/icons/package";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
  import { Badge } from "$components/ui/badge";
  import { Button } from "$components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle } from "$components/ui/card";
  import { ScrollArea } from "$components/ui/scroll-area";
  import { Separator } from "$components/ui/separator";
  import { Skeleton } from "$components/ui/skeleton";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { getProject } from "$features/projects";
  import type { Project } from "$features/projects";
  import { getSplitContent } from "$features/split";
  import { getBuild } from "$features/build";
  import { getPackage, packageEpub, verifyPackage, getPackagePath } from "$features/package";
  import type { PackageResult } from "$features/package";

  const id = $derived(page.params.id as string);
  const identifier = $derived(id ? (id.startsWith("urn:uuid:") ? id : `urn:uuid:${id}`) : "");

  let project = $state<Project | null>(null);
  let loadingProject = $state(true);
  let projectError = $state<string | null>(null);

  let splitExists = $state<boolean | null>(null);
  let loadingSplit = $state(true);

  let buildExists = $state<boolean | null>(null);
  let loadingBuild = $state(true);

  let packageData = $state<PackageResult | null>(null);
  let loadingPackage = $state(true);
  let pending = $state(false);
  let verifyPending = $state(false);

  const hasBuild = $derived(buildExists === true);
  const hasPackage = $derived(!!packageData);
  const hasChecked = $derived(packageData ? packageData.verified || packageData.issues.length > 0 : false);

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function fileIcon(name: string) {
    const ext = name.split(".").pop()?.toLowerCase() ?? "";
    if (ext === "epub") return PackageIcon;
    if (ext === "txt") return FileTextIcon;
    if (["png", "jpg", "jpeg", "webp", "gif", "bmp"].includes(ext)) return ImageIcon;
    return FileIcon;
  }

  async function loadProject() {
    if (!identifier) return;
    loadingProject = true;
    projectError = null;
    try {
      const data = await getProject(identifier);
      if (data) project = data;
      else projectError = m.workspace_load_failed();
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

  async function loadBuildCheck() {
    if (!identifier) return;
    loadingBuild = true;
    try {
      const data = await getBuild(identifier);
      buildExists = !!data;
    } catch {
      buildExists = false;
    } finally {
      loadingBuild = false;
    }
  }

  async function loadPackageData() {
    if (!identifier) return;
    loadingPackage = true;
    try {
      packageData = await getPackage(identifier);
    } catch {
      packageData = null;
    } finally {
      loadingPackage = false;
    }
  }

  onMount(() => {
    void loadProject();
    void loadSplitCheck();
    void loadBuildCheck();
    void loadPackageData();
  });

  $effect(() => {
    if (identifier) {
      void loadProject();
      void loadSplitCheck();
      void loadBuildCheck();
      void loadPackageData();
    }
  });

  async function handlePackage() {
    if (!identifier) return;
    if (!hasBuild) {
      toast.error(m.package_need_build());
      return;
    }
    pending = true;
    await tick();
    // 让 UI 先渲染 pending 状态，避免按钮卡顿
    await new Promise<void>((r) => setTimeout(r, 0));
    try {
      const res = await packageEpub(identifier);
      if (res) {
        packageData = res;
        if (res.verified) toast.success(m.package_success());
        else toast.error(m.package_verify_failed());
      } else {
        toast.error(m.package_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || m.package_failed());
    } finally {
      pending = false;
    }
  }

  async function handleVerify() {
    if (!identifier || !packageData) return;
    verifyPending = true;
    await tick();
    await new Promise<void>((r) => setTimeout(r, 0));
    try {
      const res = await verifyPackage(identifier);
      if (res) {
        packageData = res;
        if (res.verified) toast.success(m.package_verified());
        else toast.error(m.package_verify_failed());
      } else {
        toast.error(m.package_failed());
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || m.package_failed());
    } finally {
      verifyPending = false;
    }
  }

  async function handleOpenOutputs() {
    if (!packageData) return;
    try {
      const p = await getPackagePath(identifier);
      if (p) await revealItemInDir(p);
      else if (packageData.epubPath) await revealItemInDir(packageData.epubPath);
    } catch {
      toast.error(m.package_failed());
    }
  }

  function handleGoBuild() {
    void goto(resolve(`/workspace/${id}/build`));
  }

  function handleGoSplit() {
    void goto(resolve(`/workspace/${id}/split`));
  }

  const severityVariant = (s: string) => {
    const v = s.toLowerCase();
    if (v === "fatal" || v === "error") return "destructive" as const;
    if (v === "warning") return "secondary" as const;
    return "outline" as const;
  };
</script>

{#if loadingProject || loadingSplit || loadingBuild || loadingPackage}
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
      <h3 class="text-sm font-semibold">{m.package_need_build()}</h3>
      <p class="text-xs text-muted-foreground">{m.package_need_build_hint()}</p>
    </div>
    <Button size="sm" onclick={handleGoSplit}>{m.clean_go_split()}</Button>
  </div>
{:else if !hasBuild}
  <div class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center">
    <div class="space-y-2">
      <h3 class="text-sm font-semibold">{m.package_need_build()}</h3>
      <p class="text-xs text-muted-foreground">{m.package_need_build_hint()}</p>
    </div>
    <Button size="sm" onclick={handleGoBuild}>{m.package_go_build()}</Button>
  </div>
{:else if !hasPackage}
  <div class="flex flex-1 flex-col items-center justify-center overflow-auto p-6">
    <div class="flex w-full max-w-xl flex-col gap-5">
      <div class="space-y-1 text-center">
        <h3 class="text-sm font-semibold">{project?.title ?? ""}</h3>
        <p class="text-xs text-muted-foreground">{m.package_no_package_hint()}</p>
        {#if !project?.cover}
          <p class="text-xs text-muted-foreground">{m.package_no_cover_hint()}</p>
        {/if}
        <p class="text-xs text-muted-foreground">outputs/ 将包含以书名命名的 EPUB、TXT 与封面</p>
      </div>
      <Card>
        <CardContent class="space-y-3 p-4 text-xs">
          <div class="flex items-center gap-2">
            <PackageIcon class="size-4 text-muted-foreground" />
            <span class="font-medium">{m.package_file_epub()}</span>
            <span class="text-muted-foreground">· 未压缩 → EPUB3.3（mimetype 首条 Stored）</span>
          </div>
          <div class="flex items-center gap-2">
            <FileTextIcon class="size-4 text-muted-foreground" />
            <span class="font-medium">{m.package_file_txt()}</span>
            <span class="text-muted-foreground">· 含 metadata 头的纯文本</span>
          </div>
          <div class="flex items-center gap-2">
            <ImageIcon class="size-4 text-muted-foreground" />
            <span class="font-medium">{m.package_file_cover()}</span>
            <span class="text-muted-foreground">· 与 EPUB/TXT 同以书名命名</span>
          </div>
        </CardContent>
      </Card>
      <Button class="w-full gap-1.5" onclick={handlePackage} disabled={pending}>
        <PackageIcon class="size-4" />
        {pending ? m.package_packaging() : m.package_action()}
      </Button>
    </div>
  </div>
{:else if packageData}
  <div class="flex flex-1 flex-col gap-4 overflow-auto p-6">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="space-y-1">
        <h3 class="text-sm font-semibold">{packageData.bookTitle}</h3>
        <div class="flex items-center gap-2">
          {#if hasChecked}
            {#if packageData.verified}
              <Badge variant="default" class="gap-1">
                <CheckCircleIcon class="size-3" />
                {m.package_verified()}
              </Badge>
            {:else}
              <Badge variant="destructive" class="gap-1">
                <AlertCircleIcon class="size-3" />
                {m.package_verify_failed()}
              </Badge>
            {/if}
            {#if packageData.epubVersion}
              <span class="text-xs text-muted-foreground">EPUB {packageData.epubVersion}</span>
            {/if}
          {/if}
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          class="h-7 gap-1 text-xs"
          onclick={handleVerify}
          disabled={verifyPending || pending}
        >
          <ShieldCheckIcon class="size-3.5" />
          {verifyPending ? m.package_verifying() : m.package_verify()}
        </Button>
        <ConfirmDialog
          title={m.package_repackage_confirm_title()}
          message={m.package_repackage_confirm_message({ files: packageData.files.map((f) => f.name).join(", ") })}
          variant="destructive"
          confirmLabel={m.package_repackage()}
          onConfirm={handlePackage}
        >
          {#snippet trigger({ props })}
            <Button variant="outline" size="sm" class="h-7 gap-1 text-xs" {...props} disabled={pending || verifyPending}>
              <RefreshCwIcon class="size-3.5" />
              {pending ? m.package_repackaging() : m.package_repackage()}
            </Button>
          {/snippet}
        </ConfirmDialog>
        <Button variant="outline" size="sm" class="h-7 gap-1 text-xs" onclick={handleOpenOutputs}>
          <FolderIcon class="size-3.5" />
          {m.package_open_outputs()}
        </Button>
      </div>
    </div>

    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-xs font-medium">outputs/</CardTitle>
      </CardHeader>
      <CardContent class="space-y-2 p-4 pt-0">
        {#each packageData.files as f (f.path)}
          {@const Icon = fileIcon(f.name)}
          <div class="flex items-center justify-between rounded-md border px-3 py-2">
            <div class="flex min-w-0 items-center gap-2">
              <Icon class="size-4 shrink-0 text-muted-foreground" />
              <span class="truncate font-mono text-xs">{f.name}</span>
            </div>
            <span class="shrink-0 text-xs text-muted-foreground">{formatSize(f.size)}</span>
          </div>
        {:else}
          <p class="py-4 text-center text-xs text-muted-foreground">{m.package_no_files()}</p>
        {/each}
        {#if !packageData.coverPath}
          <p class="text-xs text-muted-foreground">{m.package_no_cover_hint()}</p>
        {/if}
      </CardContent>
    </Card>

    {#if hasChecked}
      {#if packageData.verified}
        <div class="flex items-center gap-2 rounded-md bg-green-50 px-3 py-2 text-xs text-green-700 dark:bg-green-950/30 dark:text-green-300">
          <CheckCircleIcon class="size-4" />
          {m.package_no_issues()}
        </div>
      {:else}
        <div class="space-y-2">
          <div class="flex items-center gap-2">
            <AlertCircleIcon class="size-4 text-destructive" />
            <h4 class="text-xs font-semibold">{m.package_issues_title({ count: packageData.issues.length })}</h4>
          </div>
          <Separator />
          <ScrollArea class="max-h-64">
            <div class="space-y-2 pr-2">
              {#each packageData.issues as issue, i (i)}
                <div class="rounded-md border px-3 py-2 text-xs">
                  <div class="flex flex-wrap items-center gap-1.5">
                    <Badge variant={severityVariant(issue.severity)} class="text-xs">{issue.severity}</Badge>
                    <span class="font-mono text-xs">{issue.id}</span>
                    {#if issue.location}
                      <span class="truncate text-muted-foreground">{issue.location}{#if issue.position}:{issue.position}{/if}</span>
                    {/if}
                  </div>
                  <p class="mt-1 text-xs leading-5">{issue.text}</p>
                  {#if issue.rule}
                    <p class="font-mono text-xs text-muted-foreground">{issue.rule}</p>
                  {/if}
                </div>
              {/each}
            </div>
          </ScrollArea>
        </div>
      {/if}
    {:else}
      <div class="flex flex-col items-center gap-3 rounded-md border border-dashed px-4 py-6">
        <ShieldCheckIcon class="size-6 text-muted-foreground" />
        <p class="text-xs text-muted-foreground">校验按需触发，点击下方按钮检查 EPUB 是否符合 EPUB 3.3 规范</p>
        <Button size="sm" class="h-7 gap-1 text-xs" onclick={handleVerify} disabled={verifyPending}>
          <ShieldCheckIcon class="size-3.5" />
          {verifyPending ? m.package_verifying() : m.package_verify()}
        </Button>
      </div>
    {/if}
  </div>
{/if}
