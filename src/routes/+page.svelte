<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { Track } from '$lib/types'
    import {
        playerState,
        toOverwrite,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, listen } from '@tauri-apps/api/event'
    import {
        BOT_ERROR,
        TRACK_ENDED,
        TRACK_LOOPED,
        TRACK_PAUSED,
        TRACK_PLAYABLE,
        TRACK_PLAYED,
        UPDATE_PLAYER
    } from '$lib/events'
    import SearchBar from '$lib/SearchBar.svelte'

    let tracks: { track: Track; mask: boolean }[] = $state([])

    async function getTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        const cachedTracks = (await store.get<Track[]>(TRACKS_SETTING)) ?? []
        tracks = cachedTracks.map((track) => {
            return { track, mask: true }
        })
    }

    function filterTracks(searchTerm: string) {
        if (searchTerm.length > 0) {
            for (const pair of tracks) {
                pair.mask = pair.track.title
                    .toLocaleLowerCase()
                    .includes(searchTerm)
            }
        } else {
            tracks.forEach((pair) => (pair.mask = true))
        }
    }

    let unlisten: UnlistenFn[] = []
    onMount(async () => {
        const toast: ToastContext = getContext('toast')

        // Setup all the global event listeners
        let unlisten1 = await listen<string>(BOT_ERROR, (ev) => {
            toast.create({
                title: 'Error',
                description: ev.payload,
                type: 'error'
            })
        })
        unlisten.push(unlisten1)

        let unlisten2 = await listen<any>(TRACK_PLAYED, (ev) => {
            if (ev.payload['is_parallel'] === false) {
                playerState.playing = true
            }
        })
        unlisten.push(unlisten2)

        let unlisten3 = await listen<any>(TRACK_PAUSED, (ev) => {
            if (ev.payload['is_parallel'] === false) {
                playerState.playing = false
            }
        })
        unlisten.push(unlisten3)

        let unlisten4 = await listen<any>(TRACK_ENDED, (ev) => {
            let isParallel = ev.payload['is_parallel'] as boolean
            let trackPath = ev.payload['path'] as string

            if (isParallel) {
                return
            }

            // If there is a track to overwrite, overwrite the current track
            // otherwise push to the end of queue
            let ended_track: Track | undefined
            if (toOverwrite.track && playerState.trackQueue[0]) {
                ended_track = playerState.trackQueue[0]
                playerState.trackQueue[0] = toOverwrite.track
            } else {
                ended_track = playerState.trackQueue.shift()
            }
            toOverwrite.track = undefined

            // Reset position
            playerState.position = 0
            // Make sure to sync play state if queue is now empty
            if (playerState.trackQueue.length === 0) {
                playerState.playing = false
            }
            // Update recent tracks if anything was removed
            if (ended_track) {
                playerState.recentlyPlayed.unshift(ended_track)
            }
        })
        unlisten.push(unlisten4)

        let unlisten5 = await listen<any>(TRACK_LOOPED, (ev) => {
            if (ev.payload['is_parallel'] === false) {
                playerState.position = 0
            }
        })
        unlisten.push(unlisten5)

        let unlisten6 = await listen<any>(TRACK_PLAYABLE, (ev) => {
            if (ev.payload['is_parallel'] === false) {
                playerState.playing = true
            }
        })
        unlisten.push(unlisten6)

        let unlisten7 = await listen<any>(UPDATE_PLAYER, (ev) => {
            playerState.position =
                ev.payload['position'] ?? playerState.position
            playerState.mute = ev.payload['mute'] ?? playerState.mute
        })
        unlisten.push(unlisten7)

        await getTracks()
    })

    onDestroy(() => {
        for (const unlistenFn of unlisten) {
            unlistenFn()
        }
    })
</script>

<div class="flex h-screen">
    <LeftSidebar {getTracks} />
    <main class="flex flex-col p-4 min-h-0 grow">
        <div class="flex grow min-h-0">
            <div class="flex grow flex-col">
                <SearchBar {filterTracks} />
                <div class="mr-4 flex flex-wrap gap-2 overflow-y-auto p-1">
                    {#each tracks as { track, mask }}
                        {#if mask}
                            <SongBox {track} />
                        {/if}
                    {:else}
                        <div class="type-scale-5">No songs!</div>
                    {/each}
                </div>
            </div>
        </div>

        <PlayerBar />
    </main>
    <RightSidebar />
</div>
