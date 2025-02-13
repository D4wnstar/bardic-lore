<script lang="ts">
    import { ChevronLeft, PenBox, Plus, Save, TagIcon, X } from 'lucide-svelte'
    import type { PageData } from './$types'
    import GroupDeleteConfirmation from '$lib/popovers/tag-editor-components/GroupDeleteConfirmation.svelte'
    import TagWithDelete from '$lib/popovers/tag-editor-components/TagWithDelete.svelte'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { getContext, onMount } from 'svelte'
    import { load, Store } from '@tauri-apps/plugin-store'
    import {
        appTags,
        appTracks,
        TAGS_FILENAME,
        TAGS_SETTING,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import type { Tag, TagGroup } from '$lib/types'
    import { TagGroupSet } from '$lib/state/taggroupset.svelte'
    import { TagSet } from '$lib/state/tagset.svelte'
    import { TrackSet } from '$lib/state/trackset.svelte'

    let { data }: { data: PageData } = $props()

    const toast: ToastContext = getContext('toast')
    let tagsStore: Store
    let tracksStore: Store

    let removeGroupModal: boolean = $state(false)
    let removeTagModalState = $state(false)
    let editingGroupName: string | null = $state(null)
    let editingGroupString: string = $state('')
    let editingTagName: string | null = $state(null)
    let editingTagString: string = $state('')

    let draggingTag: Tag | null = $state(null)
    const dragHoverClass = 'preset-outlined-surface-400-600'

    async function saveToStore() {
        await tagsStore.set(TAGS_SETTING, appTags)
        await tracksStore.set(TRACKS_SETTING, appTracks)
    }

    function handleDragStart(event: DragEvent, tag: Tag) {
        draggingTag = tag
        // Necessary to start the drag-and-drop transfer, even if we don't use it
        event.dataTransfer?.setData('text/plain', tag.value)
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

    async function handleDrop(newGroup: string) {
        if (draggingTag) {
            appTags.moveTag(newGroup, draggingTag)
            draggingTag = null
            await saveToStore()
        }
    }

    async function addRemoveTag(tag: Tag, mode: 'add' | 'remove') {
        // Update both the local tags and the global ones
        if (mode === 'add') {
            appTags.addTagOwners(tag.group, tag.value, tag.owners.tracks)
            data.track.tags.add(tag)
        } else if (mode === 'remove') {
            appTags.deleteTagOwners(tag.group, tag.value, tag.owners.tracks)
            data.track.tags.delete(tag)
        }

        await saveToStore()
    }

    async function deleteTag(tag: Tag) {
        appTags.deleteTag(tag, tag.group)
        // Since the tag is completely removed from the app, delete it from all tracks
        for (const track of appTracks) {
            track.tags.delete(tag)
        }
        await saveToStore()
    }

    async function addNewTagGroup() {
        const existingGroupNames = appTags.groups.map((g) => g.name)
        let newId = 1
        let groupName = `New Group ${newId}`

        while (existingGroupNames.includes(groupName)) {
            newId += 1
            groupName = `New Group ${newId}`
        }

        const newGroup: TagGroup = {
            name: groupName,
            tagSet: new TagSet([]),
            builtin: false,
            modifiable: true
        }
        appTags.add(newGroup)
        await saveToStore()
    }

    async function deleteTagGroup(groupName: string) {
        if (groupName !== TagGroupSet.DEFAULT_GROUP) {
            // Delete all tags in the group from all tracks in the app
            for (const tag of appTags.get(groupName)?.tagSet ?? []) {
                for (const track of appTracks) {
                    track.tags.delete(tag)
                }
            }
            appTags.delete(groupName)
            await saveToStore()
        }
    }

    function renameTagGroup(oldName: string, newName: string) {
        const okay = appTags.rename(oldName, newName)
        if (!okay) {
            toast.create({
                title: 'Error',
                description: `There is already a group called ${newName}.`,
                type: 'error'
            })
        }

        editingGroupName = null
    }

    async function addTagFromText(
        e: KeyboardEvent & {
            currentTarget: EventTarget & HTMLInputElement
        },
        groupName: string
    ) {
        if (e.code === 'Enter') {
            if (editingTagString.length > 0) {
                const newTag: Tag = {
                    value: editingTagString,
                    owners: new TrackSet([]),
                    group: groupName
                }
                appTags.addTag(groupName, newTag)
                await saveToStore()
            }

            editingTagName = null
            editingTagString = ''
        }
    }

    onMount(async () => {
        tagsStore = await load(TAGS_FILENAME)
        tracksStore = await load(TRACKS_FILENAME)
    })
</script>

<a href="/" class="flex"><ChevronLeft class="inline" />Back</a>
<div class="grow space-y-2">
    <div class="flex gap-3">
        <TagIcon class="self-center" />
        <header class="type-scale-6"><b>Tag editor</b></header>
    </div>
    <hr class="hr" />

    <p>On {data.track.title ?? data.track.filename}</p>
    <div
        class="preset-outlined-surface-200-800 p-2 rounded-md flex flex-wrap gap-1 min-h-10"
        role="listbox"
        tabindex="0"
    >
        {#each data.track.tags.sorted() as tag}
            <div role="listitem">
                <TagChip
                    {tag}
                    onclick={() => addRemoveTag(tag, 'remove')}
                    disabled={!appTags.get(tag.group)?.modifiable}
                    preset="preset-filled"
                />
            </div>
        {:else}
            <span class="opacity-40">Selected tags will be here</span>
        {/each}
    </div>

    {#each appTags.groups.filter((g) => g.modifiable) as group (group)}
        {#if !group.builtin}
            <div class="flex gap-2">
                {#if editingGroupName !== group.name}
                    <button
                        onclick={() => {
                            editingGroupName = group.name
                            editingGroupString = group.name
                        }}
                    >
                        <PenBox
                            class="self-center opacity-50 hover:opacity-100"
                            size="18"
                        />
                    </button>
                    <p>{group.name}</p>
                {:else}
                    <button
                        onclick={() =>
                            renameTagGroup(group.name, editingGroupString)}
                    >
                        <Save
                            class="self-center opacity-50 hover:opacity-100"
                            size="18"
                        />
                    </button>
                    <input
                        type="text"
                        name="group-name"
                        class="input h-6 self-center"
                        bind:value={editingGroupString}
                        onkeydown={(e) => {
                            if (e.code === 'Enter') {
                                renameTagGroup(group.name, editingGroupString)
                            }
                        }}
                    />
                {/if}
            </div>
        {:else}
            <p>{group.name}</p>
        {/if}

        <div
            class="preset-outlined-surface-200-800 p-2 rounded-md flex"
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={(_e) => handleDrop(group.name)}
            role="listbox"
            tabindex="0"
        >
            {#if group.modifiable}
                <button
                    class="mr-2 mt-0.5 opacity-50 hover:opacity-100 self-start"
                    onclick={() => (editingTagName = group.name)}
                >
                    <Plus />
                </button>
            {/if}
            <div class="grow flex flex-wrap gap-1">
                {#if editingTagName === group.name}
                    <div
                        class="flex preset-outlined-surface-200-800 rounded items-center px-2 gap-1"
                    >
                        <button
                            class="opacity-50 hover:opacity-100"
                            onclick={() => {
                                editingTagString = ''
                                editingTagName = null
                            }}
                        >
                            <X />
                        </button>
                        <input
                            type="text"
                            name="group-name"
                            class="input-ghost w-32 h-7 mr-2"
                            placeholder="New tag"
                            bind:value={editingTagString}
                            onkeydown={async (e) =>
                                await addTagFromText(e, group.name)}
                        />
                    </div>
                {/if}
                {#each group.tagSet.sorted() as tag}
                    <TagWithDelete
                        {tag}
                        open={removeTagModalState}
                        draggable={appTags.groups.filter((g) => g.modifiable)
                            .length > 1}
                        {handleDragStart}
                        {addRemoveTag}
                        {deleteTag}
                        active={data.track.tags.has(tag.value)}
                    />
                {:else}
                    <p class="opacity-40 self-center">
                        {#if group.name === TagGroupSet.DEFAULT_GROUP}
                            Click on the + to add tags
                        {:else}
                            Drag tags to change their group
                        {/if}
                    </p>
                {/each}
            </div>

            {#if !group.builtin}
                <GroupDeleteConfirmation
                    open={removeGroupModal}
                    {deleteTagGroup}
                    groupName={group.name}
                />
            {/if}
        </div>
    {/each}

    <button
        class="btn preset-tonal flex gap-2 rounded-md self-center"
        onclick={addNewTagGroup}><Plus />Add new group</button
    >
</div>
