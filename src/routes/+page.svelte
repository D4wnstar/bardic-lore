<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { CachedTrack, Track } from '$lib/types'
    import {
        appState,
        skipRemoveOnEnd,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, listen } from '@tauri-apps/api/event'
    import {
        ADD_TRACK,
        BOT_ERROR,
        CLEAR_QUEUE,
        LEFT_VOICE_CHANNEL,
        TRACK_ENDED,
        TRACK_LOOPED,
        TRACK_PAUSED,
        TRACK_PLAYABLE,
        TRACK_PLAYED,
        UPDATE_PLAYER,
        type AddTrackPayload,
        type TrackEventPayload
    } from '$lib/events'
    import SearchBar from '$lib/SearchBar.svelte'
    import { getPlayerByUuid } from '$lib/utils/utils'
    import { Player } from '$lib/state.svelte'

    let tracks: { track: CachedTrack; mask: boolean }[] = $state([])

    async function getTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        const cachedTracks =
            (await store.get<CachedTrack[]>(TRACKS_SETTING)) ?? []
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

        let unlisten2 = await listen<TrackEventPayload>(TRACK_PLAYED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.start()
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (player) player.start()
            }
        })
        unlisten.push(unlisten2)

        let unlisten3 = await listen<TrackEventPayload>(TRACK_PAUSED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.stop()
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (player) player.stop()
            }
        })
        unlisten.push(unlisten3)

        let unlisten4 = await listen<TrackEventPayload>(TRACK_ENDED, (ev) => {
            if (ev.payload.isParallel === false) {
                // If there is a track to overwrite, overwrite the current track
                // otherwise push to the end of queue
                let endedTrack: Track | undefined
                if (!skipRemoveOnEnd.skip) {
                    let res = appState.playlist.next()
                    endedTrack = res?.justEnded.track
                }
                skipRemoveOnEnd.skip = false

                // Reset position
                appState.mainPlayer.position = 0
                // Make sure to sync play state if queue is now empty
                if (appState.playlist.isEmpty()) {
                    appState.mainPlayer.stop()
                }
                // Update recent tracks if anything was removed
                if (endedTrack) {
                    appState.recentlyPlayed.unshift(endedTrack)
                }
            } else {
                const endedTrack = appState.parallelPlayers.find(
                    ({ track }) => track.uuid !== ev.payload.uuid
                )?.track
                // Delete both the track and the player, since it is no longer needed
                appState.parallelPlayers = appState.parallelPlayers.filter(
                    ({ track }) => track.uuid !== ev.payload.uuid
                )
                // Update recent tracks if anything was removed
                if (endedTrack) {
                    appState.recentlyPlayed.unshift(endedTrack)
                }
            }
        })
        unlisten.push(unlisten4)

        let unlisten5 = await listen<TrackEventPayload>(TRACK_LOOPED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.position = 0
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (player) player.position = 0
            }
        })
        unlisten.push(unlisten5)

        let unlisten6 = await listen<TrackEventPayload>(
            TRACK_PLAYABLE,
            (ev) => {
                if (ev.payload.isParallel === false) {
                    appState.mainPlayer.start()
                } else {
                    const player = getPlayerByUuid(ev.payload.uuid)
                    if (player) player.start()
                }
            }
        )
        unlisten.push(unlisten6)

        let unlisten7 = await listen<any>(UPDATE_PLAYER, (ev) => {
            if (ev.payload['uuid']) {
                const player = getPlayerByUuid(ev.payload['uuid'])
                if (!player) return
                player.position = ev.payload['position'] ?? player.position
            } else {
                appState.mainPlayer.position =
                    ev.payload['position'] ?? appState.mainPlayer.position
            }
        })
        unlisten.push(unlisten7)

        let unlisten8 = await listen<AddTrackPayload>(ADD_TRACK, (ev) => {
            if (!ev.payload.parallel) {
                if (
                    ev.payload.overwrite &&
                    appState.playlist.queue.length > 0
                ) {
                    appState.playlist.overwriteCurrent(ev.payload.track)
                } else if (ev.payload.prepend) {
                    appState.playlist.enqueuePriority(ev.payload.track)
                } else {
                    appState.playlist.enqueue(ev.payload.track)
                }
            } else {
                const player = new Player({
                    playing: false,
                    position: 0,
                    volume: appState.mainPlayer.volume,
                    looping: ev.payload.looping,
                    mute: false
                })
                appState.parallelPlayers.push({
                    track: ev.payload.track,
                    player
                })
            }
        })
        unlisten.push(unlisten8)

        let unlisten9 = await listen<any>(CLEAR_QUEUE, (ev) => {
            let current = appState.playlist.current()
            if (current) {
                appState.recentlyPlayed.push(current)
            }
            appState.playlist.clear()
            appState.mainPlayer.reset()
        })
        unlisten.push(unlisten9)

        let unlisten10 = await listen<any>(LEFT_VOICE_CHANNEL, (_ev) => {
            let current = appState.playlist.current()
            if (current) {
                appState.recentlyPlayed.push(current)
            }
            appState.playlist.clear()
            appState.mainPlayer.reset()

            for (const state of appState.parallelPlayers) {
                appState.recentlyPlayed.push(state.track)
                state.player.stop()
            }
            appState.parallelPlayers = []
        })
        unlisten.push(unlisten10)

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
