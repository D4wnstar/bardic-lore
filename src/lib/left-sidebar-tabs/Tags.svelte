<script lang="ts">
    import { TagGroupSet, TagSet } from '$lib/state.svelte'
    import { appTags } from '$lib/stores.svelte'
    import { DEFAULT_GROUP, type Tag } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Search } from 'lucide-svelte'
    import { untrack } from 'svelte'
    import { flip } from 'svelte/animate'
    import { quintOut } from 'svelte/easing'
    import { crossfade } from 'svelte/transition'

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

    let searchTerm = $state('')
    let filteredGroups = $derived.by(() => {
        return new TagGroupSet(
            appTags.groups.map((g) => {
                // The untrack is to avoid the internal state of filter
                // Derived should still run on these two variables
                selectedTags
                searchTerm
                let set = untrack(() =>
                    g.tagSet
                        .difference(selectedTags)
                        .filter((t) =>
                            t.value.toLowerCase().includes(searchTerm)
                        )
                )
                return {
                    name: g.name,
                    tagSet: set
                }
            })
        )
    })

    function handleAvailableClick(tag: Tag) {
        selectedTags.add(tag)
        filteredGroups.deleteTag(tag.group ?? DEFAULT_GROUP, tag)
        filterTracks()
    }

    function handleSelectedClick(tag: Tag) {
        filteredGroups.addTag(tag.group ?? DEFAULT_GROUP, tag)
        selectedTags.delete(tag)
        filterTracks()
    }

    const [send, receive] = crossfade({
        duration: (d) => Math.sqrt(d * 200),

        fallback(node, _params) {
            const style = getComputedStyle(node)
            const transform = style.transform === 'none' ? '' : style.transform

            return {
                duration: 600,
                easing: quintOut,
                css: (t) => `
				transform: ${transform} scale(${t});
				opacity: ${t}
			`
            }
        }
    })
</script>

<div id="tag-sidebar" class="flex flex-col h-full min-h-0">
    <h3 class="type-scale-7 heading-font-weight text-primary-900-100 px-2">
        Tags
    </h3>
    <div class="bg-surface-100-900 rounded-md flex items-center px-2 mt-2">
        <Search />
        <input
            type="search"
            bind:value={searchTerm}
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

    <div
        class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 mx-2 space-y-2"
    >
        <div
            class="preset-filled-surface-100-900 !bg-opacity-50 border-[1px] border-primary-100-900 !border-opacity-70 rounded-md p-2"
        >
            <p class="text-secondary-700-300 mb-2">
                <b>Selected</b>
            </p>
            <div class="flex flex-wrap gap-1">
                {#each selectedTags.sorted() as tag (tag)}
                    <div
                        in:receive={{ key: tag.value }}
                        out:send={{ key: tag.value }}
                        animate:flip={{ duration: 100 }}
                    >
                        <TagChip
                            {tag}
                            classes="hover:opacity-70"
                            onclick={async () => handleSelectedClick(tag)}
                        />
                    </div>
                {:else}
                    <span class="opacity-40"
                        >Select tags by clicking on them</span
                    >
                {/each}
            </div>
        </div>
        <div
            class="preset-filled-surface-100-900 !bg-opacity-50 border-[1px] border-primary-100-900 !border-opacity-70 rounded-md p-2 space-y-1"
        >
            <p class="text-secondary-700-300 mb-2">
                <b>Available</b>
            </p>
            {#if filteredGroups.size === 0 || filteredGroups.groups.every((g) => g.tagSet.size === 0)}
                <span class="opacity-40"
                    >Add tags by right clicking on tracks</span
                >
            {:else}
                {#each filteredGroups.groups.filter((g) => g.tagSet.size > 0) as group (group)}
                    <p class="pl-1 text-secondary-950-50">{group.name}</p>
                    <div class="flex flex-wrap justify-stretch gap-1">
                        {#each group.tagSet.sorted() as tag (tag)}
                            <div
                                in:receive={{ key: tag.value }}
                                out:send={{ key: tag.value }}
                                animate:flip={{ duration: 100 }}
                            >
                                <TagChip
                                    {tag}
                                    classes="hover:opacity-70"
                                    onclick={async () =>
                                        handleAvailableClick(tag)}
                                    preset="preset-tonal"
                                />
                            </div>
                        {:else}
                            <p class="opacity-40 pl-1">This group is empty</p>
                        {/each}
                    </div>
                {/each}
            {/if}
        </div>
    </div>
</div>
