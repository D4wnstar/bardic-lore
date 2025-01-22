<script lang="ts">
    import { Modal } from '@skeletonlabs/skeleton-svelte'
    import { Trash2 } from 'lucide-svelte'

    interface Props {
        open: boolean
        groupName: string
        deleteTagGroup: (groupName: string) => Promise<void>
    }

    let { open, groupName, deleteTagGroup }: Props = $props()
</script>

<Modal
    bind:open
    contentBackground="bg-surface-100-900"
    contentClasses="w-1/4 shadow-xl border-2 border-error-100-900 p-4 flex flex-col"
    backdropBase=""
>
    {#snippet trigger()}
        <Trash2 class="opacity-50 hover:opacity-100" />
    {/snippet}
    {#snippet content()}
        <p>
            This will delete the group and all the tags in it. The tags will be
            removed from all tracks that have them.
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
                    await deleteTagGroup(groupName)
                    open = false
                }}>Delete</button
            >
        </footer>
    {/snippet}
</Modal>
