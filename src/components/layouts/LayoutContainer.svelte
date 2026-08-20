<script lang="ts">
  import { settings } from "$libs/stores";
  import { layouts } from "./index";

  const { layout } = settings;

  let { children } = $props();

  // 非法/未知布局名回退默认（localStorage 残留旧值兜底）
  const Layout = $derived(layouts[$layout] ?? layouts.default);
</script>

<Layout>{@render children()}</Layout>
