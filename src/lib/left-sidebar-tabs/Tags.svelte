<script lang="ts">
    import { TagSet } from '$lib/state.svelte'
    import { appTags } from '$lib/stores.svelte'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Search } from 'lucide-svelte'

    let selectedTags: TagSet = new TagSet([])
    let availableTags = $derived(appTags.difference(selectedTags))
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
        <button class="preset-outlined-primary-400-600 btn">ALL</button>
        |
        <button class="preset-outlined-primary-400-600 btn opacity-40"
            >ANY</button
        >
    </div>

    <div class="flex-1 overflow-auto min-h-0 mx-2 space-y-2">
        <div class="bg-secondary-50-950 rounded-md p-2">
            <b>Selected</b>
            <div>
                {#each selectedTags as tag}
                    <TagChip {tag} />
                {/each}
            </div>
        </div>
        <div class="bg-tertiary-50-950 rounded-md p-2">
            <b>Available</b>
            <div class="flex flex-wrap gap-1">
                {#each availableTags as tag}
                    <TagChip {tag} />
                {/each}
            </div>
        </div>
    </div>
</div>
