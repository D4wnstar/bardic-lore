<script lang="ts">
    import { Tooltip } from '@skeletonlabs/skeleton-svelte'
    import type { Snippet } from 'svelte'
    import { slide } from 'svelte/transition'

    interface Props {
        open: boolean
        classes?: string
        contentClasses?: string
        tooltip: string
        onclick: () => void
        children: Snippet<[]>
    }

    let {
        open = $bindable(false),
        classes,
        contentClasses,
        tooltip,
        onclick,
        children
    }: Props = $props()
</script>

<Tooltip
    bind:open
    base="z-10"
    contentBase="card preset-filled-surface-100-900 border-[1px] border-primary-100-900 px-2 py-1 type-scale-2"
    {contentClasses}
    openDelay={700}
>
    {#snippet trigger()}
        <button
            class={`btn-icon rounded-none ${classes}`}
            {onclick}
            transition:slide={{ axis: 'x', duration: 100 }}
            >{@render children()}</button
        >
    {/snippet}
    {#snippet content()}
        {tooltip}
    {/snippet}
</Tooltip>
