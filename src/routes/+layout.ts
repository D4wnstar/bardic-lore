import {
    SETTINGS_FILENAME,
    appState,
    LOOP_SETTING,
    SHUFFLE_SETTING,
    MUTE_SETTING,
    settings,
    SHOW_COVERS_SETTING,
    AUTOCONNECT_SETTING,
    skipRemoveOnEnd,
    appTags,
    TAGS_FILENAME,
    TAGS_SETTING,
    AUTOHIDE_SIDEBARS_SETTING,
    SHOW_ALBUM_TAGS,
    SHOW_ARTIST_TAGS
} from '$lib/stores.svelte'
import { load as tauriLoad } from '@tauri-apps/plugin-store'
import type { LayoutLoad } from './$types'
import {
    type TrackEventPayload,
    TRACK_PLAYED,
    TRACK_PAUSED,
    TRACK_ENDED,
    CREATE_PLAYLIST,
    type CreatePlaylistPayload,
    TRACK_LOOPED,
    TRACK_PLAYABLE,
    UPDATE_PLAYER,
    type AddTrackPayload,
    ADD_TRACK,
    LEFT_VOICE_CHANNEL,
    type PlaylistCreatedPayload,
    PLAYLIST_CREATED,
    type QueueShuffledPayload,
    QUEUE_SORTED,
    QueueMethod
} from '$lib/events'
import { LoopState, Player, TagSet, TrackSet } from '$lib/state.svelte'
import {
    ALBUM_GROUP,
    ARTIST_GROUP,
    DEFAULT_GROUP,
    type TagGroup,
    type Track
} from '$lib/types'
import { listen, emit } from '@tauri-apps/api/event'

export const load = (async () => {
    // Load defaults and/or persisted states before the page loads
    const settingsStore = await tauriLoad(SETTINGS_FILENAME)
    const tagsStore = await tauriLoad(TAGS_FILENAME)

    // Player state
    appState.player.loopState =
        (await settingsStore.get(LOOP_SETTING)) ?? appState.player.loopState
    appState.player.shuffle =
        (await settingsStore.get(SHUFFLE_SETTING)) ?? appState.player.shuffle
    appState.player.mute =
        (await settingsStore.get(MUTE_SETTING)) ?? appState.player.mute

    // Settings
    settings.showCovers =
        (await settingsStore.get(SHOW_COVERS_SETTING)) ?? settings.showCovers
    settings.autoconnect =
        (await settingsStore.get(AUTOCONNECT_SETTING)) ?? settings.autoconnect
    settings.autohideSidebars =
        (await settingsStore.get(AUTOHIDE_SIDEBARS_SETTING)) ??
        settings.autohideSidebars
    settings.showAlbumTags =
        (await settingsStore.get(SHOW_ALBUM_TAGS)) ?? settings.showAlbumTags
    settings.showArtistTags =
        (await settingsStore.get(SHOW_ARTIST_TAGS)) ?? settings.showArtistTags

    // Tags
    const cachedGroups = (await tagsStore.get<TagGroup[]>(TAGS_SETTING)) ?? []

    appTags.add({ name: DEFAULT_GROUP, tagSet: new TagSet([]) })
    appTags.add({ name: ALBUM_GROUP, tagSet: new TagSet([]) })
    appTags.add({ name: ARTIST_GROUP, tagSet: new TagSet([]) })
    for (const group of cachedGroups) {
        //@ts-expect-error JavaScript has no clue how to deserialize into a class so we create TagSets manually
        group.tagSet = new TagSet(group.tagSet)
        //@ts-expect-error Same for TrackSets
        group.tagSet.tags.forEach((t) => (t.owners = new TrackSet(t.owners)))

        appTags.add(group)
    }

    // Setup all the global event listeners
    await listen<TrackEventPayload>(TRACK_PLAYED, (ev) => {
        if (ev.payload.isParallel === false) {
            appState.player.start()
        } else {
            const player = appState.parallel.getPlayerByUuid(ev.payload.uuid)
            if (player) player.start()
        }
    })

    await listen<TrackEventPayload>(TRACK_PAUSED, (ev) => {
        if (ev.payload.isParallel === false) {
            appState.player.stop()
        } else {
            const player = appState.parallel.getPlayerByUuid(ev.payload.uuid)
            if (player) player.stop()
        }
    })

    await listen<TrackEventPayload>(TRACK_ENDED, async (ev) => {
        if (ev.payload.isParallel === false) {
            // If there is a track to overwrite, overwrite the current track
            // otherwise push to the end of queue
            let endedTrack: Track | undefined
            if (!skipRemoveOnEnd.skip) {
                let res = appState.playlist.next()
                endedTrack = res?.justEnded
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
                if (!appState.offline) {
                    // If the main player is set to loop the playlist, send all the
                    // the previous tracks back to the client
                    await emit(CREATE_PLAYLIST, {
                        guildId: appState.guildId,
                        tracksData: appState.playlist.previous,
                        loopFirst: false,
                        volume: appState.player.volume,
                        shuffle: appState.player.shuffle
                    } satisfies CreatePlaylistPayload)
                }
            } else {
                appState.playlist.clear()
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
    })

    await listen<TrackEventPayload>(TRACK_LOOPED, (ev) => {
        if (ev.payload.isParallel === false) {
            appState.player.position = 0
        } else {
            const player = appState.parallel.getPlayerByUuid(ev.payload.uuid)
            if (player) player.position = 0
        }
    })

    await listen<TrackEventPayload>(TRACK_PLAYABLE, (ev) => {
        if (ev.payload.isParallel === false) {
            appState.player.start()
        } else {
            const player = appState.parallel.getPlayerByUuid(ev.payload.uuid)
            if (player) player.start()
        }
    })

    await listen<any>(UPDATE_PLAYER, (ev) => {
        if (ev.payload['uuid']) {
            const player = appState.parallel.getPlayerByUuid(ev.payload['uuid'])
            if (!player) return
            player.position = ev.payload['position'] ?? player.position
        } else {
            appState.player.position =
                ev.payload['position'] ?? appState.player.position
        }
    })

    await listen<AddTrackPayload>(ADD_TRACK, (ev) => {
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

    await listen<any>(LEFT_VOICE_CHANNEL, (_ev) => {
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

    await listen<PlaylistCreatedPayload>(PLAYLIST_CREATED, (ev) => {
        appState.playlist.clear()
        for (const track of ev.payload.tracks) {
            appState.playlist.enqueue(track)
        }
    })

    await listen<QueueShuffledPayload>(QUEUE_SORTED, (ev) => {
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
    })

    return {}
}) satisfies LayoutLoad

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

// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true
export const ssr = false
