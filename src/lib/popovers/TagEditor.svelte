<script lang="ts">
    import { TrackSet } from '$lib/state.svelte'
    import { appTags, TAGS_FILENAME, TAGS_SETTING } from '$lib/stores.svelte'
    import type { CachedTrack, Tag } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Modal } from '@skeletonlabs/skeleton-svelte'
    import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
    import { load } from '@tauri-apps/plugin-store'
    import { Tag as TagIcon, XCircle } from 'lucide-svelte'
    import { onDestroy, onMount } from 'svelte'

    interface Props {
        track: CachedTrack
        open: boolean
    }
    let { track, open = $bindable() }: Props = $props()

    let trackTags = $state(appTags.getByTrack(track))
    let availableTags = $derived(appTags.difference(trackTags))

    let removeTagModalState = $state(false)

    let draggingTag: string | null = $state(null)
    const dragHoverClass = 'preset-outlined-surface-400-600'

    function handleDragStart(event: DragEvent, tag: string) {
        draggingTag = tag
        event.dataTransfer?.setData('text/plain', tag)
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault()
        const target = event.currentTarget as HTMLElement
        target.classList.add(dragHoverClass)
    }

    function handleDragLeave(event: DragEvent) {
        const target = event.currentTarget as HTMLElement
        target.classList.remove(dragHoverClass)
    }

    async function handleDrop(event: DragEvent, outcome: 'add' | 'remove') {
        event.preventDefault()
        const target = event.currentTarget as HTMLElement
        target.classList.remove(dragHoverClass)

        if (draggingTag) {
            const tag: Tag = {
                value: draggingTag,
                owners: new TrackSet([track])
            }

            // Update both the local tags and the global ones
            if (outcome === 'add') {
                appTags.addOwners(tag)
                trackTags.add(tag)
            } else if (outcome === 'remove') {
                appTags.deleteOwners(tag)
                trackTags.delete(tag)
            }

            const store = await load(TAGS_FILENAME)
            await store.set(TAGS_SETTING, appTags)
        }
        draggingTag = null
    }

    async function handleTextInput(
        e: KeyboardEvent & {
            currentTarget: EventTarget & HTMLInputElement
        }
    ) {
        if (e.code === 'Enter') {
            const newTag: Tag = {
                value: e.currentTarget.value,
                owners: new TrackSet([track])
            }
            appTags.add(newTag)
            trackTags.add(newTag)
            e.currentTarget.value = ''

            const store = await load(TAGS_FILENAME)
            await store.set(TAGS_SETTING, appTags)
        }
    }

    async function deleteTag(tag: Tag) {
        appTags.delete(tag)

        const store = await load(TAGS_FILENAME)
        await store.set(TAGS_SETTING, appTags)

        // Emit an event to be caught by other TagEditors so that they can
        // delete this tag too, if it's in their track
        await emit('deleted-tag', tag)
    }

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        unlisten = await listen<Tag>('deleted-tag', (ev) => {
            trackTags.delete(ev.payload)
        })
    })

    onDestroy(() => {
        if (unlisten) unlisten()
    })
</script>

<Modal
    bind:open
    classes="absolute"
    contentClasses="preset-filled-surface-100-900 shadow-2xl p-4 w-1/3 h-1/2 space-y-2 overflow-auto select-none"
>
    {#snippet content()}
        <div class="flex gap-3">
            <TagIcon class="self-center" />
            <header class="type-scale-6"><b>Tag editor</b></header>
        </div>
        <hr class="hr" />
        <p>On this track</p>
        <div
            class="preset-outlined-surface-200-800 p-2 rounded-md flex flex-wrap gap-1 min-h-10"
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, 'add')}
            role="listbox"
            tabindex="0"
        >
            {#each trackTags as tag}
                <TagChip
                    draggable={true}
                    ondragstart={(e) => handleDragStart(e, tag.value)}
                    {tag}
                />
            {:else}
                <span class="opacity-40">Drag-and-drop your tags here...</span>
            {/each}
        </div>
        <p>Add new tags</p>
        <input
            class="input rounded-md"
            type="text"
            name="add-tags"
            placeholder="Write new tags here..."
            onkeypress={handleTextInput}
        />
        <p>Available</p>
        <div
            class="preset-outlined-surface-200-800 p-2 rounded-md flex flex-wrap gap-1 min-h-10"
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, 'remove')}
            role="listbox"
            tabindex="0"
        >
            {#each availableTags as tag}
                <TagChip
                    draggable={true}
                    ondragstart={(e) => handleDragStart(e, tag.value)}
                    {tag}
                >
                    {#snippet removeBtn()}
                        <Modal
                            bind:open={removeTagModalState}
                            triggerClasses="mr-1 mt-1"
                            contentBackground="bg-surface-100-900"
                            contentClasses="shadow-xl border-2 border-error-100-900 p-4 flex flex-col gap-2"
                        >
                            {#snippet trigger()}
                                <XCircle size="16" />
                            {/snippet}
                            {#snippet content()}
                                <p>
                                    This will delete the tag from all tracks
                                    that have it. <b
                                        >This action is irreversible.</b
                                    >
                                </p>
                                <footer class="self-end">
                                    <button
                                        class="btn preset-tonal"
                                        onclick={() => {
                                            removeTagModalState = false
                                        }}>Cancel</button
                                    >
                                    <button
                                        class="btn preset-tonal-error"
                                        onclick={() => {
                                            deleteTag(tag)
                                            removeTagModalState = false
                                        }}>Delete</button
                                    >
                                </footer>
                            {/snippet}
                        </Modal>
                    {/snippet}
                </TagChip>
            {:else}
                <span class="opacity-40">Create new tags in the box above</span>
            {/each}
        </div>
    {/snippet}
</Modal>
