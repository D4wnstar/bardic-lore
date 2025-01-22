<script lang="ts">
    import type { Tag } from '$lib/types'
    import type { Snippet } from 'svelte'

    interface Props {
        tag: Tag
        onclick?: () => Promise<void>
        removeBtn?: Snippet<[Tag]>
        classes?: string
        preset?: string
    }

    let {
        tag,
        onclick,
        removeBtn,
        classes,
        preset = 'preset-filled'
    }: Props = $props()
</script>

<div class="chip {preset} {classes} text-wrap" role="listitem">
    {#if removeBtn}
        {@render removeBtn(tag)}
    {/if}
    {#if onclick}
        <button onclick={async (_e) => await onclick()}>
            {tag.value}
        </button>
    {:else}
        <span>{tag.value}</span>
    {/if}
</div>
