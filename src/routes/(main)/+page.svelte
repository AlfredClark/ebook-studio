<script lang="ts">
  import { m } from "$libs/i18n/paraglide/messages";
</script>

<!-- 首页欢迎页 -->
<div class="flex h-full w-full flex-col items-center justify-center gap-6 p-6">
  <div class="flex items-center gap-6">
    <img src="/tauri.svg" alt="Tauri" class="h-20 w-auto" />
    <img src="/svelte.svg" alt="Svelte" class="h-20 w-auto" />
  </div>
  <h1 class="text-lg font-bolder">{m.home_title()}</h1>
  <p class="max-w-md text-center text-sm text-muted-foreground">{m.home_description()}</p>
</div>
