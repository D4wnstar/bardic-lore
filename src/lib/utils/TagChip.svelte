<script lang="ts">
    import type { Tag } from '$lib/types'
    import type { Snippet } from 'svelte'

    interface Props {
        tag: Tag
        draggable?: boolean
        ondragstart?: (
            ev: DragEvent & {
                currentTarget: EventTarget & HTMLDivElement
            }
        ) => void
        removeBtn?: Snippet
    }

    let { tag, draggable, ondragstart, removeBtn }: Props = $props()
</script>

<div
    class={{
        'chip preset-filled': true,
        'cursor-grab': draggable
    }}
    {draggable}
    {ondragstart}
    role="listitem"
>
    {#if removeBtn}
        {@render removeBtn()}
    {/if}
    {tag.value}
</div>
