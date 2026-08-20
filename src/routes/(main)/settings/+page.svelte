<script lang="ts">
  import AppearanceSettings from "$components/pages/settings/AppearanceSettings.svelte";
  import SystemSettings from "$components/pages/settings/SystemSettings.svelte";
  import ConfirmDialog from "$components/widgets/overlay/ConfirmDialog.svelte";
  import { Button } from "$components/ui/button";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { settings } from "$libs/stores";
  import { resetSystemConfig } from "$libs/system";
  import { setMode } from "mode-watcher";

  // 恢复默认设置（跨领域编排留在组件层，同托盘↔closeBehavior 先例）：
  // 系统配置（config.json）→ UI 偏好（localStorage）→ 暗色偏好（mode-watcher）→ 整页重载。
  // 重载让 locale 重置（跟随系统语言）与系统配置一并生效（同 initLocale 自愈模式）
  async function handleReset() {
    const config = await resetSystemConfig();
    if (!config) {
      toast.error(m.settings_reset_defaults_failed());
      return;
    }
    settings.layout.reset();
    settings.theme.reset();
    settings.closeBehavior.reset();
    settings.checkUpdates.reset();
    settings.font.reset();
    settings.fontSize.reset();
    settings.fontWeight.reset();
    setMode("system");
    location.reload();
  }
</script>

<div class="mx-auto w-full max-w-2xl space-y-8 p-6">
  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.settings_section_appearance()}</h2>
    <div class="divide-y rounded-lg border">
      <AppearanceSettings />
    </div>
  </section>

  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.settings_section_system()}</h2>
    <div class="divide-y rounded-lg border">
      <SystemSettings />
    </div>
  </section>

  <section class="space-y-3">
    <ConfirmDialog
      title={m.settings_reset_defaults_confirm_title()}
      message={m.settings_reset_defaults_confirm_message()}
      variant="destructive"
      onConfirm={() => void handleReset()}
    >
      {#snippet trigger({ props })}
        <Button variant="destructive" class="w-full" {...props}>{m.settings_reset_defaults()}</Button>
      {/snippet}
    </ConfirmDialog>
  </section>
</div>
