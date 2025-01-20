<script lang="ts">
    import { TrackSet } from '$lib/state.svelte'
    import { appTags, TAGS_FILENAME, TAGS_SETTING } from '$lib/stores.svelte'
    import type { CachedTrack, Tag } from '$lib/types'
    import { Modal } from '@skeletonlabs/skeleton-svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { Tag as TagIcon } from 'lucide-svelte'

    interface Props {
        track: CachedTrack
        open: boolean
    }
    let { track, open = $bindable() }: Props = $props()

    // We can't $derive trackTags because getByTrack mutates appTags and that's forbidden
    let trackTags = $state(appTags.getByTrack(track))
    let availableTags = $derived(appTags.difference(trackTags))

    // console.log('TRACK', track.title ?? track.filename)
    // console.log('GLOBAL:', $state.snapshot(appTags.tags))
    // console.log('TRACK:', $state.snapshot(trackTags))
    // console.log('AVAILABLE:', $state.snapshot(availableTags))
    // console.log($state.snapshot(appTags.getByValue('Rain')))
    // console.log('-------------------------')

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
</script>

<Modal
    bind:open
    classes="absolute"
    contentClasses="preset-filled-surface-100-900 shadow-2xl p-4 w-1/3 h-1/2 space-y-2 overflow-auto select-none"
>
    {#snippet content()}
        <div class="flex gap-3">
            <TagIcon class="self-center" />
            <header class="type-scale-5">Tag editor</header>
        </div>
        <hr class="hr" />
        <p>Track tags</p>
        <div
            class="preset-outlined-surface-200-800 p-2 rounded-md flex flex-wrap gap-1 min-h-10"
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, 'add')}
            role="listbox"
            tabindex="0"
        >
            {#each trackTags as tag}
                <div
                    class="chip preset-filled cursor-grab"
                    draggable="true"
                    ondragstart={(e) => handleDragStart(e, tag.value)}
                    role="listitem"
                >
                    {tag.value}
                </div>
            {:else}
                <span class="opacity-40">Drag-and-drop your tags here...</span>
            {/each}
        </div>
        <p>Existing tags</p>
        <div
            class="preset-outlined-surface-200-800 p-2 rounded-md flex flex-wrap gap-1 min-h-10"
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, 'remove')}
            role="listbox"
            tabindex="0"
        >
            {#each availableTags as tag}
                <div
                    class="chip preset-filled cursor-grab"
                    draggable="true"
                    ondragstart={(e) => handleDragStart(e, tag.value)}
                    role="listitem"
                >
                    {tag.value}
                </div>
            {/each}
        </div>
    {/snippet}
</Modal>
