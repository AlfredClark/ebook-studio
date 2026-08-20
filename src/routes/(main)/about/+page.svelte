<script lang="ts">
  import AppAbout from "$components/pages/about/AppAbout.svelte";
  import SystemAbout from "$components/pages/about/SystemAbout.svelte";
  import { m } from "$libs/i18n/paraglide/messages";
</script>

<div class="mx-auto w-full max-w-2xl space-y-8 p-6">
  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.about_section_app()}</h2>
    <div class="divide-y rounded-lg border">
      <AppAbout />
    </div>
  </section>

  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.about_section_system()}</h2>
    <div class="divide-y rounded-lg border">
      <SystemAbout />
    </div>
  </section>
</div>
