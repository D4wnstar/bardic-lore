<script lang="ts">
    import type { Tag } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Modal } from '@skeletonlabs/skeleton-svelte'
    import { XCircle } from 'lucide-svelte'

    interface Props {
        tag: Tag
        draggable: boolean
        open: boolean
        handleDragStart: (e: any, tag: Tag) => void
        addRemoveTag: (tag: Tag, mode: 'add' | 'remove') => Promise<void>
        deleteTag: (tag: Tag) => Promise<void>
    }

    let {
        tag,
        draggable,
        open,
        handleDragStart,
        addRemoveTag,
        deleteTag
    }: Props = $props()
</script>

<div {draggable} ondragstart={(e) => handleDragStart(e, tag)} role="listitem">
    <TagChip
        {tag}
        {removeBtn}
        onclick={async () => await addRemoveTag(tag, 'add')}
    />
</div>

{#snippet removeBtn(tag: Tag)}
    <Modal
        bind:open
        triggerClasses="mt-1 mr-1"
        contentBackground="bg-surface-100-900"
        contentClasses="w-1/4 shadow-xl border-2 border-error-100-900 p-4 flex flex-col gap-2"
        backdropBase=""
    >
        {#snippet trigger()}
            <XCircle size="16" />
        {/snippet}
        {#snippet content()}
            <p>
                This will delete permanently delete the tag. It will be removed
                from all tracks that have it.
                <b>This action is irreversible.</b>
            </p>
            <footer class="self-end">
                <button
                    class="btn preset-tonal"
                    onclick={() => {
                        open = false
                    }}>Cancel</button
                >
                <button
                    class="btn preset-tonal-error"
                    onclick={async () => {
                        await deleteTag(tag)
                        open = false
                    }}>Delete</button
                >
            </footer>
        {/snippet}
    </Modal>
{/snippet}
