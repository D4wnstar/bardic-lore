<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { CachedTrack, MaskedTrack, Track } from '$lib/types'
    import {
        appState,
        LOOP_SETTING,
        MUTE_SETTING,
        SETTINGS_FILENAME,
        SHUFFLE_SETTING,
        skipRemoveOnEnd,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, emit, listen } from '@tauri-apps/api/event'
    import {
        ADD_TRACK,
        BOT_ERROR,
        QUEUE_EMPTIED,
        LEFT_VOICE_CHANNEL,
        QueueMethod,
        TRACK_ENDED,
        TRACK_LOOPED,
        TRACK_PAUSED,
        TRACK_PLAYABLE,
        TRACK_PLAYED,
        UPDATE_PLAYER,
        type AddTrackPayload,
        type TrackEventPayload,
        PLAYLIST_CREATED,
        type PlaylistCreatedPayload,
        CREATE_PLAYLIST,
        type CreatePlaylistPayload,
        type QueueShuffledPayload,
        QUEUE_SORTED
    } from '$lib/events'
    import SearchBar from '$lib/SearchBar.svelte'
    import { LoopState, Player } from '$lib/state.svelte'
    import { Folder, Wind } from 'lucide-svelte'
    import { rgbToHex } from '$lib/utils/utils'
    import { fade } from 'svelte/transition'

    let iconColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-500')
    )

    let tracks: MaskedTrack[] = $state([])
    $effect(() => {
        appState.availableTracks = tracks
            .filter((t) => t.mask)
            .map((t) => t.track)
    })

    async function getCachedTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        const cachedTracks =
            (await store.get<CachedTrack[]>(TRACKS_SETTING)) ?? []
        tracks = cachedTracks
            .map((track) => {
                return { track, mask: true }
            })
            .toSorted((a, b) => a.track.title.localeCompare(b.track.title))
    }

    function addTrack(track: CachedTrack) {
        const exists = tracks.find((mt) => mt.track.path === track.path)
        if (!exists) {
            const sortedIndex = tracks.findIndex(
                (mt) => mt.track.title.localeCompare(track.title) === 1
            )
            tracks.splice(sortedIndex, 0, { track, mask: true })
        }
    }

    function removeTrack(track: CachedTrack) {
        const idxToDelete = tracks.findIndex(
            (mt) => mt.track.path === track.path
        )
        if (idxToDelete >= 0) tracks.splice(idxToDelete, 1)
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

    function handleAddTrackMain(method: QueueMethod, track: Track) {
        switch (method) {
            case QueueMethod.Normal:
                appState.playlist.enqueue(track)
                break
            case QueueMethod.Backskip:
                appState.playlist.popPrevious()
                appState.playlist.enqueueFront(track)
                break
            case QueueMethod.Priority:
                if (appState.playlist.queue.length === 0) {
                    appState.playlist.enqueue(track)
                } else {
                    appState.playlist.enqueuePriority(track)
                }
                break
            case QueueMethod.OverwriteCurrent:
                if (appState.playlist.queue.length > 0) {
                    let overwritten = appState.playlist.overwriteCurrent(track)
                    if (overwritten) {
                        appState.recentlyPlayed.unshift(overwritten)
                    }
                } else {
                    appState.playlist.enqueue(track)
                }
                break
        }
    }

    let unlisten: UnlistenFn[] = []
    onMount(async () => {
        const toast: ToastContext = getContext('toast')
        await getCachedTracks()

        // Initialize some cached settings
        const settingsStore = await load(SETTINGS_FILENAME)
        appState.player.loopState =
            (await settingsStore.get(LOOP_SETTING)) ?? appState.player.loopState
        appState.player.shuffle =
            (await settingsStore.get(SHUFFLE_SETTING)) ??
            appState.player.shuffle
        appState.player.mute =
            (await settingsStore.get(MUTE_SETTING)) ?? appState.player.mute

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
                appState.player.start()
            } else {
                const player = appState.parallel.getPlayerByUuid(
                    ev.payload.uuid
                )
                if (player) player.start()
            }
        })
        unlisten.push(unlisten2)

        let unlisten3 = await listen<TrackEventPayload>(TRACK_PAUSED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.player.stop()
            } else {
                const player = appState.parallel.getPlayerByUuid(
                    ev.payload.uuid
                )
                if (player) player.stop()
            }
        })
        unlisten.push(unlisten3)

        let unlisten4 = await listen<TrackEventPayload>(
            TRACK_ENDED,
            async (ev) => {
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
                    appState.player.position = 0
                    // Update recent tracks if anything was removed
                    if (endedTrack) {
                        appState.recentlyPlayed.unshift(endedTrack)
                    }

                    if (!appState.playlist.isEmpty()) return

                    // Make sure to handle playlist loops
                    if (appState.player.loopState === LoopState.LoopPlaylist) {
                        appState.playlist.loop()
                        if (!appState.offline) {
                            // If the main player is set to loop the playlist, send all the
                            // the tracks back to the client
                            await emit(CREATE_PLAYLIST, {
                                guildId: appState.guildId,
                                tracksData: appState.playlist.queue,
                                loopFirst: false,
                                volume: appState.player.volume,
                                shuffle: appState.player.shuffle
                            } satisfies CreatePlaylistPayload)
                        }
                    } else {
                        appState.player.stop()
                    }
                } else {
                    const endedTrack = appState.parallel.tracks.find(
                        (track) => track.uuid !== ev.payload.uuid
                    )
                    // Delete both the track and the player, since it is no longer needed
                    appState.parallel.states = appState.parallel.states.filter(
                        ({ track }) => track.uuid !== ev.payload.uuid
                    )
                    // Update recent tracks if anything was removed
                    if (endedTrack) {
                        appState.recentlyPlayed.unshift(endedTrack)
                    }
                }
            }
        )
        unlisten.push(unlisten4)

        let unlisten5 = await listen<TrackEventPayload>(TRACK_LOOPED, (ev) => {
            if (ev.payload.isParallel === false) {
                appState.player.position = 0
            } else {
                const player = appState.parallel.getPlayerByUuid(
                    ev.payload.uuid
                )
                if (player) player.position = 0
            }
        })
        unlisten.push(unlisten5)

        let unlisten6 = await listen<TrackEventPayload>(
            TRACK_PLAYABLE,
            (ev) => {
                if (ev.payload.isParallel === false) {
                    appState.player.start()
                } else {
                    const player = appState.parallel.getPlayerByUuid(
                        ev.payload.uuid
                    )
                    if (player) player.start()
                }
            }
        )
        unlisten.push(unlisten6)

        let unlisten7 = await listen<any>(UPDATE_PLAYER, (ev) => {
            if (ev.payload['uuid']) {
                const player = appState.parallel.getPlayerByUuid(
                    ev.payload['uuid']
                )
                if (!player) return
                player.position = ev.payload['position'] ?? player.position
            } else {
                appState.player.position =
                    ev.payload['position'] ?? appState.player.position
            }
        })
        unlisten.push(unlisten7)

        let unlisten8 = await listen<AddTrackPayload>(ADD_TRACK, (ev) => {
            if (!ev.payload.parallel) {
                handleAddTrackMain(ev.payload.queueMethod, ev.payload.track)
            } else {
                const player = new Player({
                    playing: false,
                    position: 0,
                    volume: appState.player.volume,
                    loopState: ev.payload.looping
                        ? LoopState.LoopTrack
                        : LoopState.None,
                    shuffle: false,
                    mute: false
                })
                appState.parallel.states.push({
                    track: ev.payload.track,
                    player
                })
            }
        })
        unlisten.push(unlisten8)

        let unlisten9 = await listen<any>(QUEUE_EMPTIED, (_ev) => {
            const current = appState.playlist.current()
            if (current) {
                appState.recentlyPlayed.push(current)
            }
            appState.playlist.clear()
            appState.player.reset()
        })
        unlisten.push(unlisten9)

        let unlisten10 = await listen<any>(LEFT_VOICE_CHANNEL, (_ev) => {
            const current = appState.playlist.current()
            if (current) {
                appState.recentlyPlayed.push(current)
            }
            for (const track of appState.parallel.tracks) {
                appState.recentlyPlayed.push(track)
            }

            appState.playlist.clear()
            appState.parallel.clear()
            appState.player.reset()
        })
        unlisten.push(unlisten10)

        let unlisten11 = await listen<PlaylistCreatedPayload>(
            PLAYLIST_CREATED,
            (ev) => {
                for (const track of ev.payload.tracks) {
                    appState.playlist.enqueue(track)
                }
            }
        )
        unlisten.push(unlisten11)

        let unlisten12 = await listen<QueueShuffledPayload>(
            QUEUE_SORTED,
            (ev) => {
                if (appState.playlist.isEmpty()) {
                    console.warn(
                        `A ${QUEUE_SORTED} event was received with an empty queue`
                    )
                    return
                }
                // Ignore the first UUID, since it's already playing
                appState.playlist.sortByUuids(ev.payload.uuids, {
                    skipFirst: true
                })
            }
        )
        unlisten.push(unlisten12)
    })

    // Uncomment to debug playlist
    // $inspect(appState.playlist.queue).with((type, queue) => {
    //     console.log('QUEUE', queue)
    //     return console.log
    // })
    // $inspect(appState.playlist.previous).with((_type, prev) => {
    //     console.log('PREVIOUS', prev)
    //     return console.log
    // })
    // $inspect(appState.playlist.priority).with((type, pri) => {
    //     console.log('PRIORITY', pri)
    //     return console.log
    // })

    onDestroy(() => {
        for (const unlistenFn of unlisten) {
            unlistenFn()
        }
    })
</script>

<div class="flex h-screen">
    <LeftSidebar {addTrack} {removeTrack} {getCachedTracks} />
    <main class="flex flex-col p-4 min-h-0 grow">
        <div class="flex grow min-h-0">
            <div class="flex grow flex-col">
                <SearchBar {filterTracks} />
                {#if tracks.length > 0}
                    <div class="mr-4 flex flex-wrap gap-2 overflow-y-auto p-1">
                        {#each appState.availableTracks as track (track.path)}
                            <div
                                class="flex-[10rem] xl:flex-[12rem] max-w-[14rem]"
                                transition:fade={{ duration: 200 }}
                            >
                                <SongBox {track} />
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
