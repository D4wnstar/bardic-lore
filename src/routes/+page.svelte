<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { Track } from '$lib/types'
    import {
        appState,
        skipRemoveOnEnd,
        TRACKS_FILENAME,
        TRACKS_SETTING,
        type PlayerState
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, listen } from '@tauri-apps/api/event'
    import {
        ADD_TRACK,
        BOT_ERROR,
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

        let unlisten2 = await listen<TrackEventPayload>(TRACK_PLAYED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.playing = true
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (!player) return
                player.playing = true
            }
        })
        unlisten.push(unlisten2)

        let unlisten3 = await listen<TrackEventPayload>(TRACK_PAUSED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.playing = false
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (!player) return
                player.playing = false
            }
        })
        unlisten.push(unlisten3)

        let unlisten4 = await listen<TrackEventPayload>(TRACK_ENDED, (ev) => {
            if (ev.payload.isParallel === false) {
                // If there is a track to overwrite, overwrite the current track
                // otherwise push to the end of queue
                let ended_track: Track | undefined
                if (skipRemoveOnEnd.skip) {
                    skipRemoveOnEnd.skip = false
                } else {
                    ended_track = appState.trackQueue.shift()
                }

                // Reset position
                appState.mainPlayer.position = 0
                // Make sure to sync play state if queue is now empty
                if (appState.trackQueue.length === 0) {
                    appState.mainPlayer.playing = false
                }
                // Update recent tracks if anything was removed
                if (ended_track) {
                    appState.recentlyPlayed.unshift(ended_track)
                }
            } else {
                appState.parallelTracks = appState.parallelTracks.filter(
                    ({ track }) => track.uuid !== ev.payload.uuid
                )
            }
        })
        unlisten.push(unlisten4)

        let unlisten5 = await listen<TrackEventPayload>(TRACK_LOOPED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.mainPlayer.position = 0
            } else {
                const player = getPlayerByUuid(ev.payload.uuid)
                if (!player) return
                player.position = 0
            }
        })
        unlisten.push(unlisten5)

        let unlisten6 = await listen<TrackEventPayload>(
            TRACK_PLAYABLE,
            (ev) => {
                if (ev.payload.isParallel === false) {
                    appState.mainPlayer.playing = true
                } else {
                    const player = getPlayerByUuid(ev.payload.uuid)
                    if (!player) return
                    player.playing = true
                }
            }
        )
        unlisten.push(unlisten6)

        let unlisten7 = await listen<any>(UPDATE_PLAYER, (ev) => {
            appState.mainPlayer.position =
                ev.payload['position'] ?? appState.mainPlayer.position
            appState.mainPlayer.mute =
                ev.payload['mute'] ?? appState.mainPlayer.mute
        })
        unlisten.push(unlisten7)

        let unlisten8 = await listen<AddTrackPayload>(ADD_TRACK, (ev) => {
            if (!ev.payload.parallel) {
                if (ev.payload.overwrite && appState.trackQueue.length > 0) {
                    appState.trackQueue[0] = ev.payload.track
                } else if (ev.payload.prepend) {
                    appState.trackQueue.unshift(ev.payload.track)
                } else {
                    appState.trackQueue.push(ev.payload.track)
                }
            } else {
                const player: PlayerState = {
                    playing: false,
                    position: 0,
                    volume: appState.mainPlayer.volume,
                    looping: ev.payload.looping,
                    mute: false
                }
                appState.parallelTracks.push({
                    track: ev.payload.track,
                    player
                })
            }
        })
        unlisten.push(unlisten8)

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
