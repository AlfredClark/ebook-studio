<script lang="ts">
  import { onMount } from "svelte";
  import { Label } from "$components/ui/label";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Switch } from "$components/ui/switch";
  import {
    getSystemConfig,
    loadSystemConfig,
    setSystemConfig,
    toggleAutostart,
    toggleNotification,
    toggleTray,
    toggleWindowState,
  } from "$libs/system";
  import { m } from "$libs/i18n/paraglide/messages";
  import { settings, type CloseBehaviorName } from "$libs/stores";

  const { closeBehavior, checkUpdates } = settings;

  // 系统配置真相源为 config.json：经共享状态 $libs/system 读取（启动期与 initLocale/
  // WindowControl 去重同一次 get_config IPC）；toggle 成功后在共享状态原位回填，
  // 消费方（如 WindowControl 托盘守卫）响应式联动。加载完成前禁用开关防竞态写回。
  onMount(() => {
    void loadSystemConfig();
  });

  /** 加载完成标志：未加载时禁用切换类控件，避免"先 toggle 后加载覆盖"竞态 */
  const loaded = $derived(getSystemConfig() !== null);

  /** 关闭行为选项：value 即 CloseBehaviorName 值域 */
  const closeBehaviorOptions = [
    { value: "ask", label: m.settings_close_behavior_ask },
    { value: "quit", label: m.settings_close_behavior_quit },
    { value: "minimize", label: m.settings_close_behavior_minimize },
  ] as const;

  /** 选中项文本（Select.Trigger 需调用方渲染；未知值回退首个选项） */
  const closeBehaviorLabel = $derived(
    closeBehaviorOptions.find((opt) => opt.value === $closeBehavior)?.label() ?? closeBehaviorOptions[0].label(),
  );

  // 托盘切换含跨领域联动（系统配置 × UI 偏好），留在组件层协调；
  // 其余 toggle 无联动，直接绑定 libs/system 的模块函数（成功回填由模块内部完成）
  /** toggle 失败时经共享状态浅拷贝整体刷新，校正 Switch 内部已翻转的显示态（bits-ui 点击即翻转内部值） */
  function refreshSystemConfigUi() {
    const config = getSystemConfig();
    if (config) setSystemConfig({ ...config });
  }

  async function handleToggleAutostart() {
    const next = await toggleAutostart();
    if (next === null) refreshSystemConfigUi();
  }

  async function handleToggleTray() {
    const next = await toggleTray();
    // 关闭托盘时最小化到托盘失去恢复入口，联动回退为弹窗询问（minimize 仅托盘开启时可选）
    if (next === false && closeBehavior.get() === "minimize") {
      closeBehavior.set("ask");
    }
    if (next === null) refreshSystemConfigUi();
  }

  async function handleToggleNotification() {
    const next = await toggleNotification();
    if (next === null) refreshSystemConfigUi();
  }

  async function handleToggleWindowState() {
    const next = await toggleWindowState();
    if (next === null) refreshSystemConfigUi();
  }

  /** 关闭行为切换：minimize 仅托盘开启时可选（SelectItem disabled 已在 UI 层约束） */
  function handleCloseBehaviorChange(value: string | undefined) {
    if (value && value !== $closeBehavior) {
      closeBehavior.set(value as CloseBehaviorName);
    }
  }

  /** 检查更新切换：UI 偏好（localStorage 真相源），同步读写无需 loaded 门禁 */
  function handleCheckUpdatesChange(checked: boolean) {
    checkUpdates.set(checked);
  }
</script>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_autostart()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_autostart_description()}</p>
  </div>
  <Switch checked={getSystemConfig()?.autostart ?? false} onCheckedChange={handleToggleAutostart} disabled={!loaded} />
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_tray()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_tray_description()}</p>
  </div>
  <Switch checked={getSystemConfig()?.tray ?? true} onCheckedChange={handleToggleTray} disabled={!loaded} />
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_close_behavior()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_close_behavior_description()}</p>
  </div>
  <Select type="single" value={$closeBehavior} onValueChange={handleCloseBehaviorChange}>
    <SelectTrigger class="w-40">
      {closeBehaviorLabel}
    </SelectTrigger>
    <SelectContent>
      {#each closeBehaviorOptions as opt (opt.value)}
        <SelectItem value={opt.value} disabled={opt.value === "minimize" && !(getSystemConfig()?.tray ?? true)}>
          {opt.label()}
        </SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_notification()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_notification_description()}</p>
  </div>
  <Switch checked={getSystemConfig()?.notification ?? false} onCheckedChange={handleToggleNotification} disabled={!loaded} />
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_window_state()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_window_state_description()}</p>
  </div>
  <Switch checked={getSystemConfig()?.windowState ?? false} onCheckedChange={handleToggleWindowState} disabled={!loaded} />
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_check_updates()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_check_updates_description()}</p>
  </div>
  <Switch checked={$checkUpdates} onCheckedChange={handleCheckUpdatesChange} />
</div>
