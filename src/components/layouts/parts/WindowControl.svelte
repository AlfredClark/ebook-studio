<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Minus, Pin, PinOff, Square, X, Maximize } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { EVENT_REQUEST_EXIT, listenEvent } from "$libs/events";
  import { getSystemConfig, isAlwaysOnTopSupported, loadSystemConfig } from "$libs/system";
  import { m } from "$libs/i18n/paraglide/messages";
  import { settings } from "$libs/stores";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import TooltipButton from "$components/widgets/overlay/TooltipButton.svelte";

  const appWindow = getCurrentWindow();
  const { closeBehavior } = settings;

  let maximized = $state(false);
  let alwaysOnTop = $state(false);
  let alwaysOnTopSupported = $state(true);
  let pinPending = $state(false);
  // 退出请求（托盘退出/Ctrl+Q）触发的确认框开窗状态：程序化控制（bind:open），
  // 与关闭按钮点击共用同一 ConfirmDialog
  let exitDialogOpen = $state(false);
  let unlistenResized: (() => void) | undefined;
  let unlistenRequestExit: (() => void) | undefined;
  let disposed = false;

  // 托盘状态守卫：经共享系统配置派生（用户切托盘后即时联动；$derived 内读值保持响应式）。
  // 探测兜底取 false（保守）——未确认托盘存在时 close 行为不隐藏窗口，
  // 避免残留的 minimize 偏好（外部改 config 关闭托盘）隐藏后无恢复入口
  const trayEnabled = $derived(getSystemConfig()?.tray ?? false);

  onMount(() => {
    // 置顶能力探测：Linux Wayland 下 GTK keep_above 无效，隐藏置顶按钮
    // （查询失败默认显示，模块内 ?? true 兜底）
    void isAlwaysOnTopSupported().then((v) => (alwaysOnTopSupported = v));
    void appWindow.isMaximized().then((v) => (maximized = v));
    void appWindow.isAlwaysOnTop().then((v) => (alwaysOnTop = v));
    void appWindow
      .onResized(async () => {
        maximized = await appWindow.isMaximized();
      })
      .then((fn) => (unlistenResized = fn));
    // 托盘状态经共享缓存加载（与 initLocale/SystemSettings 去重同一次 get_config IPC）
    void loadSystemConfig();
    // 退出请求（托盘退出菜单/Ctrl+Q）：按 closeBehavior 编排（与关闭按钮同路径）。
    // 布局注册表保证 WindowControl 单实例（同一时刻仅一个布局挂载），无重复监听。
    // 异步 resolve 晚于销毁时先判定 disposed 再决定清理或留存
    void listenEvent<void>(EVENT_REQUEST_EXIT, () => void handleExitRequest())
      .then((fn) => {
        if (disposed) fn();
        else unlistenRequestExit = fn;
      })
      .catch(() => {});
  });

  onDestroy(() => {
    disposed = true;
    unlistenResized?.();
    unlistenRequestExit?.();
  });

  async function toggleAlwaysOnTop() {
    // in-flight 守卫：pending 期间丢弃重复点击，避免乐观翻转与异步回读交错产生竞态
    if (pinPending) return;
    pinPending = true;
    try {
      // 以实际状态取反，不依赖本地乐观值（可能已被外部/先前操作改变）
      const target = !(await appWindow.isAlwaysOnTop());
      await appWindow.setAlwaysOnTop(target);
      alwaysOnTop = await appWindow.isAlwaysOnTop();
    } catch {
      // 失败回读实际状态兜底（尽力而为，回读失败保持现状）
      alwaysOnTop = await appWindow.isAlwaysOnTop().catch(() => alwaysOnTop);
    } finally {
      pinPending = false;
    }
  }

  // 关闭行为（非 ask 分支）：minimize 且托盘存在时隐藏到托盘（托盘点击可恢复）；
  // 托盘未确认存在（共享配置未加载/探测失败/被外部关闭）时降级为直接关闭，防止无恢复入口
  function handleClose() {
    if (closeBehavior.get() === "minimize" && trayEnabled) {
      void appWindow.hide();
    } else {
      void appWindow.close();
    }
  }

  // 退出请求（托盘退出菜单/Ctrl+Q）：先显示聚焦（窗口可能隐藏于托盘，弹窗不可见），
  // 再按 closeBehavior 走关闭按钮同一套流程——ask 打开确认框，其余复用 handleClose
  async function handleExitRequest() {
    await appWindow.show().catch(() => {});
    await appWindow.setFocus().catch(() => {});
    if ($closeBehavior === "ask") {
      exitDialogOpen = true;
    } else {
      handleClose();
    }
  }
</script>

<div class="flex h-8 items-center">
  {#if alwaysOnTopSupported}
    <TooltipButton
      label={m.window_control_pin()}
      onclick={toggleAlwaysOnTop}
      disabled={pinPending}
      class="h-8 w-11 rounded-none"
    >
      {#if alwaysOnTop}
        <PinOff class="size-4" />
      {:else}
        <Pin class="size-4" />
      {/if}
    </TooltipButton>
  {/if}
  <TooltipButton label={m.window_control_minimize()} onclick={() => void appWindow.minimize()} class="h-8 w-11 rounded-none">
    <Minus class="size-4" />
  </TooltipButton>
  <TooltipButton
    label={m.window_control_maximize()}
    onclick={() => void appWindow.toggleMaximize()}
    class="h-8 w-11 rounded-none"
  >
    {#if maximized}
      <Square class="size-4" />
    {:else}
      <Maximize class="size-4" />
    {/if}
  </TooltipButton>
  <!-- 关闭行为（ask 弹窗询问 / quit 直接关闭 / minimize 最小化到托盘）：
       关闭按钮即 AlertDialog 触发器，确认后关闭窗口；ask 经双委托（Tooltip + AlertDialog）
       由 TooltipButton 的 extraProps 内部合并，调用方无感知 -->
  {#if $closeBehavior === "ask"}
    <ConfirmDialog
      bind:open={exitDialogOpen}
      title={m.window_control_close_confirm_title()}
      message={m.window_control_close_confirm_message()}
      variant="destructive"
      onConfirm={() => void appWindow.close()}
    >
      {#snippet trigger({ props })}
        <TooltipButton
          label={m.window_control_close()}
          extraProps={props}
          class="h-8 w-11 rounded-none hover:bg-destructive hover:text-white dark:hover:bg-destructive dark:hover:text-white"
        >
          <X class="size-4" />
        </TooltipButton>
      {/snippet}
    </ConfirmDialog>
  {:else}
    <TooltipButton
      label={m.window_control_close()}
      onclick={handleClose}
      class="h-8 w-11 rounded-none hover:bg-destructive hover:text-white dark:hover:bg-destructive dark:hover:text-white"
    >
      <X class="size-4" />
    </TooltipButton>
  {/if}
</div>
