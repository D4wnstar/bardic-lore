<script lang="ts">
    import type { Tag } from '$lib/types'
    import { XCircle } from 'lucide-svelte'

    interface Props {
        tag: Tag
        draggable?: boolean
        ondragstart?: (
            ev: DragEvent & {
                currentTarget: EventTarget & HTMLDivElement
            }
        ) => void
        showRemove?: boolean
    }

    let { tag, draggable, ondragstart, showRemove }: Props = $props()
</script>

<div
    class={{
        'chip preset-filled': true,
        'cursor-grab': draggable,
        'cursor-default': !draggable
    }}
    {draggable}
    {ondragstart}
    role="listitem"
>
    {#if showRemove}
        <button class="">
            <XCircle size="16" class="mr-1" />
        </button>
    {/if}
    {tag.value}
</div>
