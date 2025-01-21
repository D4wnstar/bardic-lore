<script lang="ts">
    import { TagSet } from '$lib/state.svelte'
    import { appTags } from '$lib/stores.svelte'
    import type { Tag } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Search } from 'lucide-svelte'

    interface Props {
        selectedTags: TagSet
        filterTracks: () => void
        tagsMode: 'any' | 'all'
    }

    let {
        tagsMode = $bindable('all'),
        selectedTags = $bindable(new TagSet([])),
        filterTracks
    }: Props = $props()

    let availableTags = $derived(appTags.difference(selectedTags))

    function handleAvailableClick(tag: Tag) {
        selectedTags.add(tag)
        availableTags.delete(tag)
        filterTracks()
    }

    function handleSelectedClick(tag: Tag) {
        availableTags.add(tag)
        selectedTags.delete(tag)
        filterTracks()
    }
</script>

<div id="tag-sidebar" class="flex flex-col h-full min-h-0">
    <h3 class="type-scale-7 heading-font-weight text-primary-900-100 px-2">
        Tags
    </h3>
    <div class="bg-surface-100-900 rounded-md flex items-center px-2 mt-2">
        <Search />
        <input
            type="search"
            class="h-12 w-full border-none bg-transparent focus:ring-0"
            placeholder="Search tags..."
        />
    </div>

    <div class="my-2 flex items-center justify-center gap-2 flex-none">
        <button
            class={{
                'preset-outlined-primary-400-600 btn': true,
                'opacity-40': tagsMode === 'any'
            }}
            onclick={(_) => {
                tagsMode = 'all'
                filterTracks()
            }}>ALL</button
        >
        |
        <button
            class={{
                'preset-outlined-primary-400-600 btn': true,
                'opacity-40': tagsMode === 'all'
            }}
            onclick={(_) => {
                tagsMode = 'any'
                filterTracks()
            }}>ANY</button
        >
    </div>

    <div class="flex-1 overflow-auto min-h-0 mx-2 space-y-2">
        <div
            class="preset-filled-surface-100-900 !bg-opacity-50 border-[1px] border-primary-100-900 !border-opacity-70 rounded-md p-2"
        >
            <p class="text-secondary-700-300 mb-2">
                <b>Selected</b>
            </p>
            <div class="flex flex-wrap gap-1">
                {#each selectedTags as tag}
                    <button
                        class="cursor-pointer hover:opacity-70"
                        onclick={(_e) => handleSelectedClick(tag)}
                    >
                        <TagChip {tag} />
                    </button>
                {:else}
                    <span class="opacity-40"
                        >Select tags by clicking on them</span
                    >
                {/each}
            </div>
        </div>
        <div
            class="preset-filled-surface-100-900 !bg-opacity-50 border-[1px] border-primary-100-900 !border-opacity-70 rounded-md p-2"
        >
            <p class="text-secondary-700-300 mb-2">
                <b>Available</b>
            </p>
            <div class="flex flex-wrap gap-1">
                {#each availableTags as tag}
                    <button
                        class="cursor-pointer hover:opacity-70"
                        onclick={(_e) => handleAvailableClick(tag)}
                    >
                        <TagChip {tag} />
                    </button>
                {:else}
                    <span class="opacity-40"
                        >Add tags by right clicking on tracks</span
                    >
                {/each}
            </div>
        </div>
    </div>
</div>
