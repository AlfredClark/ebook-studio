<script lang="ts">
  import { arch, family, locale, platform, type as osType, version } from "@tauri-apps/plugin-os";
  import { onMount } from "svelte";
  import { Label } from "$components/ui/label";
  import { m } from "$libs/i18n/paraglide/messages";

  // 加载失败兜底占位符（非 Tauri 环境浏览器调试时同步 API 抛 TypeError）
  const fallback = "—";

  /** 系统信息行定义（不含值）：key 与值表 values 对齐，label 为 Paraglide 消息函数 */
  const rowDefs = [
    { key: "platform", label: m.about_system_platform },
    { key: "version", label: m.about_system_version },
    { key: "type", label: m.about_system_type },
    { key: "arch", label: m.about_system_arch },
    { key: "family", label: m.about_system_family },
    { key: "locale", label: m.about_system_locale },
  ] as const;

  /** 系统信息值表：onMount 加载真实值覆盖（$state 深层响应，行渲染跟随更新） */
  const values = $state<Record<string, string>>({});

  /** 展示行：label + 值（未加载/失败时为占位符） */
  const rows = $derived(rowDefs.map((row) => ({ ...row, value: values[row.key] ?? fallback })));

  onMount(() => {
    // 同步 API 非 Tauri 环境抛 TypeError，须逐个捕获兜底（不阻断其余行）
    const syncReads: Array<[string, () => string]> = [
      ["platform", platform],
      ["version", version],
      ["type", osType],
      ["arch", arch],
      ["family", family],
    ];
    for (const [key, read] of syncReads) {
      try {
        values[key] = read();
      } catch {
        values[key] = fallback;
      }
    }
    // locale 为异步 API：失败同样回退占位符
    void locale()
      .then((value) => {
        values.locale = value || fallback;
      })
      .catch(() => {
        values.locale = fallback;
      });
  });
</script>

{#each rows as row (row.key)}
  <div class="flex items-center justify-between gap-4 px-4 py-4">
    <Label>{row.label()}</Label>
    <span class="text-sm text-muted-foreground">{row.value}</span>
  </div>
{/each}
