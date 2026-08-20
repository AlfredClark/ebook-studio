<script lang="ts">
  import { onMount, tick } from "svelte";
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
  import { Button } from "$components/ui/button";
  import * as Command from "$components/ui/command";
  import { Label } from "$components/ui/label";
  import * as Popover from "$components/ui/popover";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$components/ui/select";
  import { Slider } from "$components/ui/slider";
  import { loadSystemFonts } from "$libs/utils";
  import { changeLocale, getLocale, type LocaleMode } from "$libs/i18n";
  import { m } from "$libs/i18n/paraglide/messages";
  import { settings, type FontSizeName, type FontWeightName, type LayoutName, type ThemeName } from "$libs/stores";
  import { getSystemConfig } from "$libs/system";
  import { themeNames } from "$styles/themes";
  import { setMode, userPrefersMode } from "mode-watcher";

  const { layout, theme, font, fontSize, fontWeight } = settings;

  /** "跟随系统"项标识：须为非空 value（cmdk/bits-ui 空 value 在搜索过滤时会被隐藏） */
  const SYSTEM_FONT_VALUE = "system-default";

  /** 语言选项：value 为 config.json 的 locale 存储模式值（"system" 跟随系统，或后端 Locale 校验域的具体标签） */
  const localeOptions = [
    { value: "system", label: m.language_system },
    { value: "en", label: m.language_en },
    { value: "zh-CN", label: m.language_zh_cn },
  ] as const;

  /** 颜色模式选项：value 即 mode-watcher Mode 值域 */
  const colorOptions = [
    { value: "system", label: m.theme_system },
    { value: "light", label: m.theme_light },
    { value: "dark", label: m.theme_dark },
  ] as const;

  /** 布局选项：value 即 LayoutName 值域 */
  const layoutOptions = [
    { value: "default", label: m.layout_default },
    { value: "dashboard", label: m.layout_dashboard },
  ] as const;

  /** 主题文案映射：paraglide 消息为具体函数对象无法动态索引，保留显式映射（与 themeNames 一一对应）；
   *  satisfies 约束：新增主题漏加 label 时编译期报错 */
  const themeLabels = {
    neutral: m.theme_neutral,
    stone: m.theme_stone,
    zinc: m.theme_zinc,
    mauve: m.theme_mauve,
    olive: m.theme_olive,
    mist: m.theme_mist,
    taupe: m.theme_taupe,
  } as const satisfies Record<ThemeName, () => string>;

  /** 主题选项：value 由 themeNames 驱动（单一真相源），label 取显式映射 */
  const themeOptions = themeNames.map((name) => ({ value: name, label: themeLabels[name] }));

  /** 字体大小选项：首项"默认"（value "" = 跟随默认），其余 px 档位（与 settings.ts fontSizeNames 一致） */
  const fontSizeOptions = [
    { value: "", label: () => m.settings_font_size_default() },
    { value: "14", label: () => m.settings_font_size_value({ size: "14" }) },
    { value: "16", label: () => m.settings_font_size_value({ size: "16" }) },
    { value: "18", label: () => m.settings_font_size_value({ size: "18" }) },
    { value: "20", label: () => m.settings_font_size_value({ size: "20" }) },
  ] as const;

  /** 字体粗细语义名映射：paraglide 消息为具体函数对象无法动态索引，保留显式映射（与 9 档一一对应） */
  const fontWeightLabels = {
    "100": m.settings_font_weight_thin,
    "200": m.settings_font_weight_extra_light,
    "300": m.settings_font_weight_light,
    "400": m.settings_font_weight_regular,
    "500": m.settings_font_weight_medium,
    "600": m.settings_font_weight_semibold,
    "700": m.settings_font_weight_bold,
    "800": m.settings_font_weight_extrabold,
    "900": m.settings_font_weight_black,
  } as const;

  // 语言存储模式值：config 快照的 locale（"system" 或具体标签），快照未加载前回退 paraglide 具体值。
  // 经 $derived 内 getSystemConfig() 读值保持响应式跟踪（快照加载/重置后联动）
  const storedLocale = $derived(getSystemConfig()?.locale ?? getLocale());

  // 系统字体列表（按家族名去重、排序）；空数组表示加载失败或进行中
  let fonts = $state<string[]>([]);
  let fontsLoading = $state(true);

  // 字体选择 Combobox 状态：Popover 开关、搜索词、触发按钮 ref（选择后回焦便于键盘续操作）
  let fontPopoverOpen = $state(false);
  let fontQuery = $state("");
  let fontTriggerRef = $state<HTMLButtonElement | null>(null);

  // 选中项文本（Select.Trigger 需调用方渲染；未知值回退首个选项）
  const localeLabel = $derived(localeOptions.find((opt) => opt.value === storedLocale)?.label() ?? localeOptions[0].label());
  const colorLabel = $derived(
    colorOptions.find((opt) => opt.value === userPrefersMode.current)?.label() ?? colorOptions[0].label(),
  );
  const layoutLabel = $derived(layoutOptions.find((opt) => opt.value === $layout)?.label() ?? layoutOptions[0].label());
  const themeLabel = $derived(themeOptions.find((opt) => opt.value === $theme)?.label() ?? themeOptions[0].label());
  const fontLabel = $derived(fontsLoading ? m.settings_font_loading() : $font === "" ? m.settings_font_system() : $font);
  const fontSizeLabel = $derived(fontSizeOptions.find((opt) => opt.value === $fontSize)?.label() ?? fontSizeOptions[0].label());
  /** 当前字重数值与语义名（滑块 value 为 number[]，偏好存字符串） */
  const fontWeightValue = $derived(Number($fontWeight));
  const fontWeightLabel = $derived(fontWeightLabels[$fontWeight]());

  /** Command.Root 选中值：与 Item.value 对齐驱动勾选标记（空偏好映射为"跟随系统"项 value） */
  const selectedFontValue = $derived($font === "" ? SYSTEM_FONT_VALUE : $font);

  /** 加载系统字体列表：完成后若已存偏好不在列表中（字体已卸载/残留），静默回退系统默认 */
  onMount(() => {
    void loadSystemFonts().then((list) => {
      fonts = list;
      fontsLoading = false;
      if ($font && !list.includes($font)) font.set("");
    });
  });

  /** 语言切换：乐观更新选中态 → changeLocale（后端落盘成功才切前端并 reload）；失败回滚 */
  async function handleLocaleChange(value: string | undefined) {
    if (!value || value === storedLocale) return;
    // 成功后 changeLocale 内部 reload 重载页面（共享快照随新 config 刷新）；
    // 失败时 derived 自动保持原存储值，无需回滚
    await changeLocale(value as LocaleMode);
  }

  /** 颜色模式切换：写 mode-watcher 偏好（userPrefersMode 自动持久化，.dark class 由 ModeWatcher 应用） */
  function handleColorChange(value: string | undefined) {
    if (value && value !== userPrefersMode.current) {
      setMode(value as "light" | "dark" | "system");
    }
  }

  /** 布局切换：直接写前端偏好 store，布局容器订阅自动切换 */
  function handleLayoutChange(value: string | undefined) {
    if (value && value !== $layout) layout.set(value as LayoutName);
  }

  /** 主题切换：直接写前端偏好 store，data-theme 应用由 storeDef subscribe 声明式注入 */
  function handleThemeChange(value: string | undefined) {
    if (value && value !== $theme) theme.set(value as ThemeName);
  }

  /** 字体选择：直接写前端偏好 store（"跟随系统"项映射回空串），关闭弹层并回焦触发按钮 + 重置搜索词 */
  function handleFontSelect(value: string) {
    font.set(value === SYSTEM_FONT_VALUE ? "" : value);
    fontQuery = "";
    fontPopoverOpen = false;
    tick().then(() => fontTriggerRef?.focus());
  }

  /** 字体大小切换：直接写前端偏好 store，html font-size 应用由 storeDef subscribe 声明式注入（空串=默认） */
  function handleFontSizeChange(value: string | undefined) {
    if (value !== undefined && value !== $fontSize) fontSize.set(value as FontSizeName);
  }

  /** 字体粗细切换：滑块即时生效（onValueChange 即写偏好 store，--app-font-weight 由 storeDef subscribe 应用） */
  function handleFontWeightChange(value: number) {
    const weight = String(value) as FontWeightName;
    if (weight !== $fontWeight) fontWeight.set(weight);
  }
</script>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_language()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_language_description()}</p>
  </div>
  <Select type="single" value={storedLocale} onValueChange={handleLocaleChange}>
    <SelectTrigger class="w-40">
      {localeLabel}
    </SelectTrigger>
    <SelectContent>
      {#each localeOptions as opt (opt.value)}
        <SelectItem value={opt.value}>{opt.label()}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_color_scheme()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_color_scheme_description()}</p>
  </div>
  <Select type="single" value={userPrefersMode.current} onValueChange={handleColorChange}>
    <SelectTrigger class="w-40">
      {colorLabel}
    </SelectTrigger>
    <SelectContent>
      {#each colorOptions as opt (opt.value)}
        <SelectItem value={opt.value}>{opt.label()}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_theme()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_theme_description()}</p>
  </div>
  <Select type="single" value={$theme} onValueChange={handleThemeChange}>
    <SelectTrigger class="w-40">
      {themeLabel}
    </SelectTrigger>
    <SelectContent class="max-h-72">
      {#each themeOptions as opt (opt.value)}
        <SelectItem value={opt.value}>{opt.label()}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_layout()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_layout_description()}</p>
  </div>
  <Select type="single" value={$layout} onValueChange={handleLayoutChange}>
    <SelectTrigger class="w-40">
      {layoutLabel}
    </SelectTrigger>
    <SelectContent>
      {#each layoutOptions as opt (opt.value)}
        <SelectItem value={opt.value}>{opt.label()}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_font()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_font_description()}</p>
  </div>
  <Popover.Root bind:open={fontPopoverOpen}>
    <Popover.Trigger bind:ref={fontTriggerRef}>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="outline"
          class="w-40 justify-between"
          role="combobox"
          aria-expanded={fontPopoverOpen}
          disabled={fontsLoading}
        >
          <span class="truncate">{fontLabel}</span>
          <ChevronsUpDownIcon class="size-4 shrink-0 opacity-50" />
        </Button>
      {/snippet}
    </Popover.Trigger>
    <Popover.Content class="w-72 p-0" align="end">
      <Command.Root value={selectedFontValue}>
        <Command.Input bind:value={fontQuery} placeholder={m.settings_font_search()} />
        <Command.List>
          <Command.Empty>{m.settings_font_empty()}</Command.Empty>
          <Command.Group>
            <Command.Item value={SYSTEM_FONT_VALUE} onSelect={() => handleFontSelect(SYSTEM_FONT_VALUE)}>
              {m.settings_font_system()}
            </Command.Item>
            {#each fonts as name (name)}
              <!-- 预览：选项文本以其自身字体渲染（style:font-family 挂在 DOM 元素上，家族名已由 libs/utils 清洗） -->
              <Command.Item value={name} onSelect={() => handleFontSelect(name)}>
                <span style:font-family={name}>{name}</span>
              </Command.Item>
            {/each}
          </Command.Group>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Root>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_font_size()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_font_size_description()}</p>
  </div>
  <Select type="single" value={$fontSize} onValueChange={handleFontSizeChange}>
    <SelectTrigger class="w-40">
      {fontSizeLabel}
    </SelectTrigger>
    <SelectContent>
      {#each fontSizeOptions as opt (opt.value)}
        <SelectItem value={opt.value}>{opt.label()}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
</div>

<div class="flex items-center justify-between gap-4 px-4 py-4">
  <div class="space-y-1">
    <Label>{m.settings_font_weight()}</Label>
    <p class="text-sm text-muted-foreground">{m.settings_font_weight_description()}</p>
  </div>
  <div class="flex w-56 items-center gap-4">
    <span class="w-16 shrink-0 text-right text-sm tabular-nums">{fontWeightLabel}</span>
    <Slider type="single" min={100} max={900} step={100} value={fontWeightValue} onValueChange={handleFontWeightChange} />
  </div>
</div>
