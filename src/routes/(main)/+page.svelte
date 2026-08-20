<script lang="ts">
  import { m } from "$libs/i18n/paraglide/messages";
</script>

<!-- 首页欢迎页 -->
<div class="flex h-full w-full flex-col items-center justify-center gap-6 p-6">
  <img src="/icon.png" alt="Ebook Studio" class="h-20 w-auto" />
  <h1 class="text-lg font-bolder">{m.home_title()}</h1>
  <p class="max-w-md text-center text-sm text-muted-foreground">{m.home_description()}</p>
</div>
