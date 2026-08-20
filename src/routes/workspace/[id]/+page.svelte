<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import { onMount } from "svelte";

  onMount(() => {
    const id = page.params.id as string | undefined;
    if (id) {
      void goto(resolve(`/workspace/${id}/inspect`), { replaceState: true });
    } else {
      void goto(resolve("/projects"));
    }
  });
</script>

<div class="flex flex-1 items-center justify-center p-6 text-sm text-muted-foreground">Redirecting…</div>
