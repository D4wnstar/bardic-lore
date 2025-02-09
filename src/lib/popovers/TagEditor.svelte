<script lang="ts">
    import { appTags, TAGS_FILENAME, TAGS_SETTING } from '$lib/stores.svelte'
    import { type CachedTrack, type Tag, type TagGroup } from '$lib/types'
    import TagChip from '$lib/utils/TagChip.svelte'
    import { Modal, type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
    import { load, Store } from '@tauri-apps/plugin-store'
    import { PenBox, Plus, Save, Tag as TagIcon, X } from 'lucide-svelte'
    import { getContext, onDestroy, onMount, untrack } from 'svelte'
    import GroupDeleteConfirmation from './tag-editor-components/GroupDeleteConfirmation.svelte'
    import TagWithDelete from './tag-editor-components/TagWithDelete.svelte'
    import { TagGroupSet } from '$lib/state/taggroupset.svelte'
    import { TagSet } from '$lib/state/tagset.svelte'
    import { TrackSet } from '$lib/state/trackset.svelte'

    interface Props {
        track: CachedTrack
        open: boolean
        handleTagEdit?: (tag: Tag, outcome: 'add' | 'remove') => void
    }
    let { track, open = $bindable(), handleTagEdit }: Props = $props()

    let trackTags = $state(appTags.getByTrack(track))
    let filteredGroups = $derived.by(() => {
        // Ignore unmodifiable groups, remove already selected tags, sort alphabetically
        const filtered: TagGroup[] = appTags.groups
            .filter((g) => g.modifiable)
            .map((g) => {
                return {
                    ...g,
                    tagSet: g.tagSet.difference(trackTags)
                }
            })
            .toSorted((a, b) => a.name.localeCompare(b.name))

        // Default group should always be last (it's guaranteed to exist)
        const defaultIndex = filtered.findIndex(
            (g) => g.name === TagGroupSet.DEFAULT_GROUP
        )
        filtered.push(...filtered.splice(defaultIndex, 1))

        return new TagGroupSet(filtered)
    })

    let draggingTag: Tag | null = $state(null)
    const dragHoverClass = 'preset-outlined-surface-400-600'

    let removeGroupModal: boolean = $state(false)
    let removeTagModalState = $state(false)
    let editingGroupName: string | null = $state(null)
    let editingGroupString: string = $state('')
    let editingTagName: string | null = $state(null)
    let editingTagString: string = $state('')
    $effect(() => {
        if (open === false) {
            untrack(() => {
                removeGroupModal = false
                removeTagModalState = false
                editingGroupName = null
                editingTagName = null
                editingTagString = ''
            })
        }
    })

    const toast: ToastContext = getContext('toast')
    let store: Store

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
            await store.set(TAGS_SETTING, appTags)
        }
    }

    async function addRemoveTag(tag: Tag, mode: 'add' | 'remove') {
        tag.owners.add(track)

        // Update both the local tags and the global ones
        if (mode === 'add') {
            appTags.addTagOwners(tag.group, tag)
            trackTags.add(tag)
            if (handleTagEdit) handleTagEdit(tag, 'add')
        } else if (mode === 'remove') {
            appTags.deleteTagOwners(tag.group, tag)
            trackTags.delete(tag)
            if (handleTagEdit) handleTagEdit(tag, 'remove')
        }

        await store.set(TAGS_SETTING, appTags)
    }

    async function deleteTag(tag: Tag) {
        appTags.deleteTag(tag.group, tag)
        if (handleTagEdit) handleTagEdit(tag, 'remove')
        await store.set(TAGS_SETTING, appTags)

        // Emit an event to be caught by other TagEditors so that they can
        // delete this tag too, if it's in their track
        await emit('deleted-tag', tag)
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
        await store.set(TAGS_SETTING, appTags)
    }

    async function deleteTagGroup(groupName: string) {
        if (groupName !== TagGroupSet.DEFAULT_GROUP) {
            for (const tag of appTags.get(groupName)?.tagSet ?? []) {
                if (handleTagEdit) handleTagEdit(tag, 'remove')
            }
            appTags.delete(groupName)
            await store.set(TAGS_SETTING, appTags)
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
                await store.set(TAGS_SETTING, appTags)
                console.log('Added tag', newTag.value, 'to group', groupName)
            }

            editingTagName = null
            editingTagString = ''
        }
    }

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        store = await load(TAGS_FILENAME)

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
    contentClasses="preset-filled-surface-100-900 shadow-2xl p-4 w-1/3 h-3/4 space-y-2 overflow-auto select-none flex flex-col"
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
            role="listbox"
            tabindex="0"
        >
            {#each trackTags.sorted() as tag}
                <div role="listitem">
                    <TagChip
                        {tag}
                        onclick={() => addRemoveTag(tag, 'remove')}
                        disabled={!appTags.get(tag.group)?.modifiable}
                    />
                </div>
            {:else}
                <span class="opacity-40">Selected tags will be here</span>
            {/each}
        </div>

        {#each filteredGroups as group (group)}
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
                                    renameTagGroup(
                                        group.name,
                                        editingGroupString
                                    )
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
                            draggable={filteredGroups.size > 1}
                            {handleDragStart}
                            {addRemoveTag}
                            {deleteTag}
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
    {/snippet}
</Modal>
