<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import {
        ALBUM_GROUP,
        ARTIST_GROUP,
        type CachedTrack,
        type MaskedTrack,
        type Tag
    } from '$lib/types'
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
    import { BOT_ERROR, CLIENT_CONNECTED } from '$lib/events'
    import SearchBar from '$lib/SearchBar.svelte'
    import { Folder, Wind } from 'lucide-svelte'
    import { createDiscordClient, rgbToHex } from '$lib/utils/utils'
    import { fade } from 'svelte/transition'
    import Fuse from 'fuse.js'
    import VirtualList from '$lib/utils/VirtualList.svelte'
    import { TrackSet } from '$lib/state/trackset.svelte'
    import { TagSet } from '$lib/state/tagset.svelte'

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

        // Reset album and artist groups to refresh tags based on available tracks
        appTags.delete(ALBUM_GROUP)
        appTags.add({
            name: ALBUM_GROUP,
            tagSet: new TagSet([]),
            builtin: true,
            modifiable: false
        })
        appTags.delete(ARTIST_GROUP)
        appTags.add({
            name: ARTIST_GROUP,
            tagSet: new TagSet([]),
            builtin: true,
            modifiable: false
        })

        tracks = cachedTracks
            .map((track) => {
                // TODO: This function makes refreshing tracks laggy: improve this
                if (settings.hideOst && track.album) {
                    track.album = track.album
                        .replace(/:? *\(?Complete[^:()]*?Soundtrack\)?/i, '')
                        .replace(/:? *\(?Deluxe[^:()]*?Soundtrack\)?/i, '')
                        .replace(/:? *\(?Original[^:()]*?Score\)?/i, '')
                        .replace(/:? *\(?Original[^:()]*?Soundtrack\)?/i, '')
                }

                if (settings.showAlbumTags && track.album) {
                    const newTag: Tag = {
                        value: track.album,
                        owners: new TrackSet([track]),
                        group: ALBUM_GROUP
                    }
                    if (appTags.getTag(track.album)) {
                        appTags.addTagOwners(ALBUM_GROUP, newTag)
                    } else {
                        appTags.addTag(ALBUM_GROUP, newTag)
                    }
                }

                if (settings.showArtistTags && track.artist) {
                    const newTag: Tag = {
                        value: track.artist,
                        owners: new TrackSet([track]),
                        group: ARTIST_GROUP
                    }
                    if (appTags.getTag(track.artist)) {
                        appTags.addTagOwners(ARTIST_GROUP, newTag)
                    } else {
                        appTags.addTag(ARTIST_GROUP, newTag)
                    }
                }

                return { track, visible: true }
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
            tracks.splice(sortedIndex, 0, { track, visible: true })
        }
    }

    function removeTrack(track: CachedTrack) {
        const idxToDelete = tracks.findIndex(
            (mt) => mt.track.path === track.path
        )
        if (idxToDelete >= 0) tracks.splice(idxToDelete, 1)
    }

    function filterTracks() {
        let timeBefore = Date.now()
        // First, set all tracks to not be visible
        tracks.forEach((pair) => (pair.visible = false))

        // Then, run string similarity search with Fuse
        let searchedTracks: MaskedTrack[]
        if (searchTerm.length > 0) {
            const fuse = new Fuse(tracks, {
                keys: ['track.title', 'track.filename'],
                threshold: 0.3
            })
            searchedTracks = fuse.search(searchTerm).map((res) => res.item)
        } else {
            searchedTracks = tracks
        }

        // Finally, filter by presence of tags
        for (const pair of searchedTracks) {
            let foundTag = true

            if (selectedTags.size > 0) {
                const trackTags = appTags.getByTrack(pair.track)
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

            pair.visible = foundTag
        }

        let timeAfter = Date.now()
        console.log(`Filtering took ${timeAfter - timeBefore} ms`)
    }

    let unlisten: UnlistenFn[] = []
    onMount(async () => {
        const toast: ToastContext = getContext('toast')
        await getCachedTracks()

        const unlisten1 = await listen<string>(BOT_ERROR, (ev) => {
            toast.create({
                title: 'Error',
                description: ev.payload,
                type: 'error'
            })
        })

        const unlisten2 = await listen<undefined>(CLIENT_CONNECTED, () => {
            toast.create({
                title: 'Created client',
                description:
                    'Successfully created client. Servers should refresh in a moment.',
                type: 'success'
            })
        })
        unlisten.push(unlisten1)
        unlisten.push(unlisten2)

        if (settings.autoconnect) {
            await createDiscordClient(toast)
        }
    })

    onDestroy(() => {
        unlisten.forEach((fn) => fn())
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
                    <VirtualList items={tracks.filter((pair) => pair.visible)}>
                        {#snippet children(pair)}
                            {#key pair}
                                <div transition:fade={{ duration: 100 }}>
                                    <SongBox track={pair.track} {tracks} />
                                </div>
                            {/key}
                        {/snippet}
                    </VirtualList>
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
