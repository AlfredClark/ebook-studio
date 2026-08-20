<script lang="ts">
  // 功能演示页：演示模板已集成的桌面能力（greet IPC + SQLite 本地持久化 + 文件拖放），
  // 后续能力演示在此按节追加。文件拖放能力层 libs/drag-drop 为通用能力，不在移除清单内。
  //
  // 【模板初始化·移除本页指引】删除下列触点即可整体移除演示模块：
  //   1. src/routes/(main)/demo/+page.svelte（本路由页）
  //   2. src/components/pages/demo/（本目录）
  //   3. src/features/demo/（前端业务模块）
  //   4. src/libs/sql/（SQL 能力层，当前仅承载演示，可随 demo 一并移除）
  //   5. src/components/layouts/parts/nav-items.ts 的 demo 导航项
  //   6. messages/{en,zh-CN}.json 中 nav_demo / home_* 之外的 demo_* 键（home_* 属首页文案勿删）
  //   7. src-tauri/src/features/demo.rs 与 src-tauri/src/commands/demo.rs
  //   8. src-tauri/src/features/mod.rs 的 `pub mod demo;` 与 commands/mod.rs 的 demo 命令注册
  //   9. src-tauri/locales/{en,zh-CN}.yml 的 `demo:` 块
  //  10. src-tauri 的 SQL 接线：Cargo.toml 的 tauri-plugin-sql 依赖、lib.rs 的插件注册行、
  //      capabilities/plugins.json 的 sql:default / sql:allow-execute 权限
  //   收尾：bun run i18n:compile && bun run validate
  import { onDestroy, onMount } from "svelte";
  import { Button } from "$components/ui/button";
  import { Input } from "$components/ui/input";
  import { listenFileDrop } from "$libs/drag-drop";
  import { greet } from "$features/demo";
  import { m } from "$libs/i18n/paraglide/messages";
  import { toast } from "$libs/overlay";
  import { initSql, insertGreetLog, listGreetLogs, type GreetLog } from "$libs/sql";

  // greet 演示：名称输入 + 按钮触发（无表单）；结果经 toast 展示
  // （成功为后端 rust-i18n 本地化文案，失败走统一错误提示）。
  // 成功后经 libs/sql 落库 greet 记录（记录失败仅提示，不影响问候展示）
  let name = $state("");
  let pending = $state(false);

  // greet 记录演示：SQLite 持久化查询（记录经前端写入，后端仅插件壳）。
  // 挂载时自动加载；发送问候成功后自动刷新联动
  let logs = $state<GreetLog[]>([]);
  let logsLoading = $state(false);

  async function loadLogs() {
    logsLoading = true;
    try {
      logs = (await listGreetLogs()) ?? [];
    } finally {
      logsLoading = false;
    }
  }

  async function handleGreet() {
    if (pending) return;
    pending = true;
    try {
      const input = name.trim();
      const greeting = await greet(input);
      if (greeting) {
        toast.success(greeting);
        if (input) {
          const saved = await insertGreetLog(input, greeting);
          if (!saved) toast.error(m.demo_greet_logs_save_failed());
          await loadLogs();
        }
      } else {
        toast.error(m.demo_greet_failed());
      }
    } finally {
      pending = false;
    }
  }

  // 文件拖放演示：窗口级监听（libs/drag-drop 封装核心 API，免权限）。
  // enter/over 驱动高亮（拖入/悬停），drop 取 paths 展示，leave 恢复
  let dragging = $state(false);
  let droppedPaths = $state<string[]>([]);
  let unlistenFileDrop: (() => void) | undefined;
  let disposedFileDrop = false;

  onMount(() => {
    // initSql 幂等；失败由列表空态自然呈现（logsLoading 结束且 logs 为空）
    void initSql().then((ok) => {
      if (!ok) toast.error(m.demo_greet_logs_failed());
    });
    void loadLogs();
    // 文件拖放监听异步 resolve：若组件已销毁，立即清理而非留存泄漏
    void listenFileDrop((event) => {
      switch (event.payload.type) {
        case "enter":
        case "over":
          dragging = true;
          break;
        case "drop":
          droppedPaths = event.payload.paths;
          dragging = false;
          break;
        case "leave":
          dragging = false;
          break;
      }
    })
      .then((fn) => {
        if (disposedFileDrop) fn();
        else unlistenFileDrop = fn;
      })
      .catch(() => {});
  });

  onDestroy(() => {
    disposedFileDrop = true;
    unlistenFileDrop?.();
  });
</script>

<div class="mx-auto w-full max-w-2xl space-y-8 p-6">
  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.demo_greet_title()}</h2>
    <div class="divide-y rounded-lg border p-4">
      <div class="flex items-center gap-2">
        <Input bind:value={name} placeholder={m.demo_greet_placeholder()} class="w-56" />
        <Button onclick={handleGreet} disabled={pending}>{m.demo_greet_button()}</Button>
      </div>
    </div>
  </section>

  <section class="space-y-3">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-bolder">{m.demo_greet_logs_title()}</h2>
      <Button variant="outline" size="sm" onclick={() => void loadLogs()} disabled={logsLoading}>
        {m.demo_greet_logs_refresh()}
      </Button>
    </div>
    <div class="divide-y rounded-lg border">
      {#if logs.length === 0}
        <p class="p-4 text-sm text-muted-foreground">{m.demo_greet_logs_empty()}</p>
      {:else}
        {#each logs as log (log.id)}
          <div class="flex items-center gap-3 px-4 py-2 text-sm">
            <span class="w-16 shrink-0 text-muted-foreground">#{log.id}</span>
            <span class="w-24 shrink-0 truncate font-medium">{log.name}</span>
            <span class="min-w-0 flex-1 truncate">{log.greeting}</span>
            <span class="shrink-0 text-muted-foreground">{log.created_at}</span>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class="space-y-3">
    <h2 class="text-lg font-bolder">{m.demo_drag_drop_title()}</h2>
    <div
      class="flex min-h-32 flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed p-6 transition-colors {dragging
        ? 'border-primary bg-accent'
        : 'border-border'}"
    >
      {#if dragging}
        <p class="text-sm font-medium">{m.demo_drag_drop_drop_hint()}</p>
      {:else}
        <p class="text-sm text-muted-foreground">{m.demo_drag_drop_hint()}</p>
      {/if}
      {#if droppedPaths.length > 0}
        <ul class="w-full space-y-1">
          {#each droppedPaths as path (path)}
            <li class="truncate text-xs text-muted-foreground" title={path}>{path}</li>
          {/each}
        </ul>
      {:else}
        <p class="text-xs text-muted-foreground">{m.demo_drag_drop_empty()}</p>
      {/if}
    </div>
  </section>
</div>
