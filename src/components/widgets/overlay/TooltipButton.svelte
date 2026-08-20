<script lang="ts">
  // 提示按钮复合组件：Tooltip 三件套 + Button 一体化封装。
  // 触发按钮经 bits-ui child 委托注入 tooltip 行为；extraProps 用于双委托场景
  // （如 ConfirmDialog 的触发按钮）吸收外部委托 props，内部经 mergeProps 链式合并
  // ref/事件（勿用对象展开覆盖）；onclick 同样走合并，undefined 时不影响外部行为。
  // restProps 置于 mergeProps 之前展开：委托属性（id/data-state/disabled/ref）优先，
  // 避免调用方散传 props 覆盖双委托属性破坏 aria 关联。
  import type { Snippet } from "svelte";
  import { mergeProps } from "svelte-toolbelt";
  import { Button, type ButtonVariant } from "$components/ui/button";
  import { Tooltip, TooltipContent, TooltipTrigger } from "$components/ui/tooltip";

  let {
    label,
    onclick,
    extraProps = {},
    variant = "ghost",
    class: className,
    children,
    ...restProps
  }: {
    /** 提示文案：TooltipContent 与 aria-label 共用 */
    label: string;
    onclick?: () => void;
    /** 双委托外部 props（如 AlertDialog 触发器委托），内部与 tooltip props 合并 */
    extraProps?: Record<string, unknown>;
    variant?: ButtonVariant;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  } = $props();
</script>

<Tooltip ignoreNonKeyboardFocus={true}>
  <TooltipTrigger>
    {#snippet child({ props: tooltipProps })}
      <Button
        {...restProps}
        {...mergeProps(tooltipProps, extraProps, { onclick })}
        {variant}
        class={className}
        aria-label={label}
      >
        {@render children?.()}
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>{label}</TooltipContent>
</Tooltip>
