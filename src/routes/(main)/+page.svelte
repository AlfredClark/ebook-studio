<script lang="ts">
  import { m } from "$libs/i18n/paraglide/messages";
</script>

<!-- 静态欢迎页：模板介绍。演示功能已迁移至 /demo（greet 见 components/pages/demo），此处不承载业务逻辑 -->
<div class="flex h-full w-full flex-col items-center justify-center gap-6 p-6">
  <div class="flex items-center gap-6">
    <img src="/tauri.svg" alt="Tauri" class="h-20 w-auto" />
    <img src="/svelte.svg" alt="Svelte" class="h-20 w-auto" />
  </div>
  <h1 class="text-lg font-bolder">{m.home_title()}</h1>
  <p class="max-w-md text-center text-sm text-muted-foreground">{m.home_description()}</p>
</div>
