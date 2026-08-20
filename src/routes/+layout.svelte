<script lang="ts">
  import { goto } from "$app/navigation";
  import { dev } from "$app/environment";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import { initLocale } from "$libs/i18n";
  import { initLogger } from "$libs/logger";
  import { logBoundaryError } from "$libs/errors";
  import { notify } from "$libs/notifications";
  import { getSystemConfig } from "$libs/system";
  import { Toaster } from "$components/ui/sonner";
  import { TooltipProvider } from "$components/ui/tooltip";
  import { ModeWatcher } from "mode-watcher";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { settings } from "$libs/stores";
  import { checkUpdateSilently } from "$libs/updater";
  import "../styles/app.css";

  // SPA 无服务端 hooks：app.html 硬编码 lang="en"，此处同步实际语言（initLocale 内部
  // 一并更新 document.documentElement.lang）。同步在 onMount（而非 layout load）执行：
  // Tauri IPC 内部使用 window.fetch，在 load 阶段会触发 SvelteKit dev 的 fetch 检查警告
  // （误报）。首帧已按 localStorage 持久化的 locale 渲染正确；initLocale 兜底自愈——
  // 与 config.json 一致时仅同步 lang 属性，失同步（外部改配置）时以 config 为准 reload 一次
  onMount(async () => {
    await initLogger();
    await initLocale();
    // 启动静默检查更新（设置开关控制）：发现新版本弹持久 toast（必须手动关闭，
    // 「查看更新」跳转关于页走安装流程）并发送系统通知（notification 开关控制）；
    // 检查失败仅记日志，不做任何 UI 反馈。dev 环境跳过（更新器未签名不可用，仅落错误日志）。
    // 置于 initLocale 之后保证文案语言正确；失同步 reload 场景最多重复检查一次（可接受）。
    // 设置经 settings.checkUpdates.get() 同步读取（localStorage 真相源）
    if (!dev && settings.checkUpdates.get()) {
      const available = await checkUpdateSilently();
      if (available) {
        toast(m.updater_update_available(), {
          description: m.updater_update_available_desc({ version: available.version }),
          duration: Infinity,
          dismissible: true,
          action: {
            label: m.updater_view_update(),
            onClick: () => void goto(resolve("/about")),
          },
        });
        // 系统通知（跨领域联动：更新 × notification 配置开关，与 toast 并存；发送失败静默）
        if (getSystemConfig()?.notification) {
          void notify(m.updater_update_available(), m.updater_update_available_desc({ version: available.version }));
        }
      }
    }
  });
</script>

<!-- 渲染边界：子组件渲染错误 → 写入日志 + 回退提示（手动重试，不自动重挂载） -->
<svelte:boundary onerror={logBoundaryError}>
  <ModeWatcher />
  <TooltipProvider delayDuration={1000}>
    <slot />
  </TooltipProvider>
  <Toaster />
  {#snippet failed(error, reset)}
    <div class="boundary-error">
      <p>{m.boundary_error({ message: error instanceof Error ? error.message : String(error) })}</p>
      <button onclick={reset}>{m.boundary_retry()}</button>
    </div>
  {/snippet}
</svelte:boundary>

<style lang="css">
  .boundary-error {
    margin: 2rem auto;
    padding: 1rem 2rem;
    max-width: 480px;
    text-align: center;
    color: #b91c1c;
  }

  .boundary-error button {
    margin-top: 0.75rem;
  }
</style>
