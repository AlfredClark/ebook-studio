<script lang="ts">
  // 确认对话框（复合组件式）：调用方经 `{#snippet trigger()}` 传入真实触发按钮（必传），
  // 内部经 bits-ui child 委托注入开窗行为（onclick/data 属性/ref），无隐藏触发器。
  // 确认按钮须显式关窗（bits-ui Action 不自动关窗）；取消/ESC/遮罩由 bits-ui 关窗；
  // confirmed 标志区分确认与取消路径，避免确认关窗触发的 onOpenChange(false) 误触 onCancel。
  import type { Snippet } from "svelte";
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
  } from "$components/ui/alert-dialog";
  import { buttonVariants } from "$components/ui/button";
  import { m } from "$libs/i18n/paraglide/messages";

  let {
    trigger,
    open = $bindable(false),
    title,
    message,
    variant = "default",
    confirmLabel,
    cancelLabel,
    onConfirm,
    onCancel,
  }: {
    /** 触发按钮 snippet：接收 bits-ui 委托的 props（须 {...props} 展开，勿覆盖 onclick） */
    trigger: Snippet<[{ props: Record<string, unknown> }]>;
    open?: boolean;
    title: string;
    message: string;
    variant?: "default" | "destructive";
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm?: () => void;
    onCancel?: () => void;
  } = $props();

  /** 危险操作时确认按钮的 destructive 变体类（经 twMerge 覆盖 Action 默认变体，保持有效 HTML） */
  const destructiveVariantClass = buttonVariants({ variant: "destructive" });

  let confirmed = $state(false);
</script>

<AlertDialog
  bind:open
  onOpenChange={(next) => {
    // 重开时重置标志；关闭且非确认路径（取消按钮/ESC/遮罩点击）触发 onCancel
    if (next) {
      confirmed = false;
    } else if (!confirmed) {
      onCancel?.();
    }
  }}
>
  <AlertDialogTrigger>
    {#snippet child({ props })}
      {@render trigger({ props })}
    {/snippet}
  </AlertDialogTrigger>
  <AlertDialogContent interactOutsideBehavior="close">
    <AlertDialogHeader>
      <AlertDialogTitle>{title}</AlertDialogTitle>
      <AlertDialogDescription>{message}</AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>
        {cancelLabel ?? m.common_cancel()}
      </AlertDialogCancel>
      <AlertDialogAction
        class={variant === "destructive" ? destructiveVariantClass : undefined}
        onclick={() => {
          confirmed = true;
          // bits-ui Action 不自动关窗，须显式关；onOpenChange(false) 到达时 confirmed 已置位，不会误触 onCancel
          open = false;
          onConfirm?.();
        }}
      >
        {confirmLabel ?? m.common_confirm()}
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
