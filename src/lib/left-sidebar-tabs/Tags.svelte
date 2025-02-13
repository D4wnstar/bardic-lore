<script lang="ts">
    import {
        appState,
        appTags,
        appTracks,
        GROUP_ACCORDION_STATES,
        selectedTags,
        SETTINGS_FILENAME
    } from '$lib/stores.svelte'
    import { type Tag } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { ChevronDown, ChevronUp, Search } from 'lucide-svelte'
    import { onMount } from 'svelte'
    import { flip } from 'svelte/animate'
    import { SvelteMap } from 'svelte/reactivity'
    import { fade, slide } from 'svelte/transition'

    let searchTerm = $state('')

    // svelte-ignore state_referenced_locally (It only needs to be initialized)
    let groupsOpen: SvelteMap<string, boolean> = $state(
        new SvelteMap(appTags.groups.map((g) => [g.name, false]))
    )

    function handleAvailableClick(tag: Tag) {
        if (selectedTags.has(tag.value)) {
            selectedTags.delete(tag)
        } else {
            selectedTags.add(tag)
        }
        // filteredGroups.deleteTag(tag, tag.group)
        appTracks.filter(
            selectedTags,
            appState.tagsMode,
            appState.trackSearchTerm
        )
    }

    function handleSelectedClick(tag: Tag) {
        // filteredGroups.addTag(tag.group, tag)
        selectedTags.delete(tag)
        appTracks.filter(
            selectedTags,
            appState.tagsMode,
            appState.trackSearchTerm
        )
    }

    async function openCloseTagGroup(groupName: string) {
        const currState = groupsOpen.get(groupName)
        groupsOpen.set(groupName, !currState)

        const store = await load(SETTINGS_FILENAME)
        await store.set(GROUP_ACCORDION_STATES, groupsOpen)
    }

    onMount(async () => {
        const store = await load(SETTINGS_FILENAME)
        const maybeCachedStates = await store.get<Map<string, boolean>>(
            GROUP_ACCORDION_STATES
        )
        if (maybeCachedStates) {
            groupsOpen = new SvelteMap(Object.entries(maybeCachedStates))
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
                'opacity-40': appState.tagsMode === 'any'
            }}
            onclick={(_) => {
                appState.tagsMode = 'all'
            }}>ALL</button
        >
        |
        <button
            class={{
                'preset-outlined-primary-400-600 btn': true,
                'opacity-40': appState.tagsMode === 'all'
            }}
            onclick={(_) => {
                appState.tagsMode = 'any'
            }}>ANY</button
        >
    </div>

    <div
        class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 mx-2 space-y-2 pb-5"
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
                        transition:fade={{ duration: 100 }}
                        animate:flip={{ duration: 100 }}
                    >
                        <TagChip
                            {tag}
                            classes="hover:opacity-70"
                            onclick={async () => handleSelectedClick(tag)}
                            preset="preset-filled"
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
            {#if appTags.size === 0 || appTags.groups.every((g) => g.tagSet.size === 0)}
                <span class="opacity-40"
                    >Add tags by right clicking on tracks</span
                >
            {:else}
                {#each appTags
                    .sorted()
                    .filter((g) => g.tagSet.size > 0) as group (group)}
                    <button
                        class="flex gap-1 pl-1 text-secondary-950-50"
                        onclick={async () =>
                            await openCloseTagGroup(group.name)}
                    >
                        {group.name}
                        {#if groupsOpen.get(group.name) === true}
                            <ChevronUp class="opacity-50" />
                        {:else}
                            <ChevronDown class="opacity-50" />
                        {/if}
                    </button>

                    {#if groupsOpen.get(group.name) === true}
                        <div
                            class="flex flex-wrap justify-stretch gap-1"
                            transition:slide={{ axis: 'y' }}
                        >
                            {#each group.tagSet.sorted() as tag (tag)}
                                <div
                                    transition:fade={{ duration: 100 }}
                                    animate:flip={{ duration: 100 }}
                                >
                                    <TagChip
                                        {tag}
                                        classes="hover:opacity-70"
                                        onclick={async () =>
                                            handleAvailableClick(tag)}
                                        preset={selectedTags.has(tag.value)
                                            ? 'preset-filled'
                                            : 'preset-tonal'}
                                    />
                                </div>
                            {:else}
                                <p class="opacity-40 pl-1">
                                    This group is empty
                                </p>
                            {/each}
                        </div>
                    {/if}
                {/each}
            {/if}
        </div>
    </div>
</div>
