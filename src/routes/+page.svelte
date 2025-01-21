<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { CachedTrack, MaskedTrack } from '$lib/types'
    import {
        appTags,
        settings,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, listen } from '@tauri-apps/api/event'
    import { BOT_ERROR } from '$lib/events'
    import SearchBar from '$lib/SearchBar.svelte'
    import { Folder, Wind } from 'lucide-svelte'
    import { createDiscordClient, rgbToHex } from '$lib/utils/utils'
    import { fade } from 'svelte/transition'
    import { TagSet } from '$lib/state.svelte'

    let iconColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-500')
    )

    let tracks: MaskedTrack[] = $state([])
    let searchTerm: string = $state('')
    let selectedTags: TagSet = $state(new TagSet([]))
    let tagsMode: 'any' | 'all' = $state('all')

    async function getCachedTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        const cachedTracks =
            (await store.get<CachedTrack[]>(TRACKS_SETTING)) ?? []
        tracks = cachedTracks
            .map((track) => {
                return { track, mask: true }
            })
            .toSorted((a, b) => {
                const titleA = a.track.title ?? a.track.filename
                const titleB = b.track.title ?? b.track.filename
                return titleA.localeCompare(titleB)
            })
    }

    function addTrack(track: CachedTrack) {
        const exists = tracks.find((mt) => mt.track.path === track.path)
        if (!exists) {
            const sortedIndex = tracks.findIndex((mt) => {
                const titleA = mt.track.title ?? mt.track.filename
                const titleB = track.title ?? track.filename
                return titleA.localeCompare(titleB) === 1
            })
            tracks.splice(sortedIndex, 0, { track, mask: true })
        }
    }

    function removeTrack(track: CachedTrack) {
        const idxToDelete = tracks.findIndex(
            (mt) => mt.track.path === track.path
        )
        if (idxToDelete >= 0) tracks.splice(idxToDelete, 1)
    }

    function filterTracks() {
        for (const pair of tracks) {
            let foundSearchTerm = true
            if (searchTerm.length > 0) {
                const title = pair.track.title ?? pair.track.filename
                foundSearchTerm = title.toLocaleLowerCase().includes(searchTerm)
            }

            let foundTag = true
            const trackTags = appTags.getByTrack(pair.track)
            if (selectedTags.size > 0) {
                if (tagsMode === 'all') {
                    foundTag = selectedTags.tags.every((tag) =>
                        trackTags.has(tag)
                    )
                } else if (tagsMode === 'any') {
                    foundTag = selectedTags.tags.some((tag) =>
                        trackTags.has(tag)
                    )
                }
            }

            pair.mask = foundSearchTerm && foundTag
        }
    }

    let unlisten: UnlistenFn
    onMount(async () => {
        const toast: ToastContext = getContext('toast')
        await getCachedTracks()

        if (settings.autoconnect) {
            await createDiscordClient(toast)
        }

        unlisten = await listen<string>(BOT_ERROR, (ev) => {
            toast.create({
                title: 'Error',
                description: ev.payload,
                type: 'error'
            })
        })
    })

    onDestroy(() => {
        if (unlisten) unlisten()
    })
</script>

<div class="flex h-screen">
    <LeftSidebar
        bind:selectedTags
        bind:tagsMode
        {addTrack}
        {removeTrack}
        {getCachedTracks}
        {filterTracks}
    />
    <main class="flex flex-col p-4 min-h-0 grow">
        <div class="flex grow min-h-0">
            <div class="flex grow flex-col">
                <SearchBar bind:searchTerm {filterTracks} />
                {#if tracks.length > 0}
                    <div class="mr-4 flex flex-wrap gap-2 overflow-y-auto p-1">
                        {#each tracks.filter((t) => t.mask) as { track } (track)}
                            <div
                                class="flex-[10rem] xl:flex-[12rem] max-w-[14rem]"
                                transition:fade={{ duration: 200 }}
                            >
                                <SongBox {track} {tracks} />
                            </div>
                        {/each}
                    </div>
                {:else}
                    <div
                        class="type-scale-6 text-surface-800-200 text-center flex flex-col gap-2 justify-center items-center h-full"
                    >
                        <p>It's a little empty here...</p>
                        <p>
                            Use the <Folder class="inline mx-1" /> sidebar on the
                            left to add some music!
                        </p>
                        <Wind size="144" color={iconColor} />
                    </div>
                {/if}
            </div>
        </div>
        <PlayerBar />
    </main>
    <RightSidebar />
</div>
