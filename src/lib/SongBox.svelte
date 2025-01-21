<script lang="ts">
    import { appState, settings, skipRemoveOnEnd } from './stores.svelte'
    import type { CachedTrack, MaskedTrack } from './types'
    import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
    import {
        PLAY_PARALLEL,
        QUEUE_TRACK,
        QueueMethod,
        type PlayParallelPayload,
        type QueueTrackPayload,
        CREATE_PLAYLIST,
        type CreatePlaylistPayload
    } from '$lib/events'
    import ContextMenu from './popovers/ContextMenu.svelte'
    import { Layers, Plus, Replace, Tag } from 'lucide-svelte'
    import { LoopState } from './state.svelte'
    import { getCover, permuteTracks } from './utils/utils'
    import { onDestroy, onMount } from 'svelte'
    import TagEditor from './popovers/TagEditor.svelte'

    interface Props {
        track: CachedTrack
        tracks: MaskedTrack[]
    }

    let { track, tracks }: Props = $props()

    let showTagEditor = $state(false)
    let showContextMenu = $state(false)
    let contextMenuX = $state(0)
    let contextMenuY = $state(0)
    let coverImage: string | undefined = $state()

    function handleContextMenu(event: MouseEvent) {
        event.preventDefault()
        contextMenuX = event.clientX
        contextMenuY = event.clientY
        showContextMenu = true
    }

    async function createPlaylist() {
        if (appState.offline) return
        // This is guaranteed to work because the track needs to be in the list
        // for us to even click on it
        const tracksToSend = permuteTracks(
            track,
            tracks.filter((mt) => mt.mask).map((mt) => mt.track)
        ) as CachedTrack[]
        console.log(tracksToSend)
        await emit(CREATE_PLAYLIST, {
            guildId: appState.guildId,
            tracksData: tracksToSend,
            volume: appState.player.volume,
            loopFirst: appState.player.loopState === LoopState.LoopTrack,
            shuffle: appState.player.shuffle
        } satisfies CreatePlaylistPayload)
    }

    async function addToQueue(method: QueueMethod) {
        if (!appState.offline) {
            if (appState.playlist.current()) {
                skipRemoveOnEnd.skip = method === QueueMethod.OverwriteCurrent
            }
            await emit(QUEUE_TRACK, {
                guildId: appState.guildId,
                trackData: track,
                looping: appState.player.loopState === LoopState.LoopTrack,
                queueMethod: method,
                volume: appState.player.volume
            } satisfies QueueTrackPayload)
        }
    }

    async function playParallel(looping: boolean) {
        if (!appState.offline) {
            await emit(PLAY_PARALLEL, {
                guildId: appState.guildId,
                trackData: track,
                volume: appState.player.volume,
                looping
            } satisfies PlayParallelPayload)
        }
    }

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        const loadCover = async () => {
            if (settings.showCovers) {
                coverImage = await getCover('cover', track.coverHash)
            } else {
                coverImage = undefined
            }
        }

        await loadCover()
        unlisten = await listen('reload-cover', loadCover)
    })

    onDestroy(() => {
        if (unlisten) unlisten()
    })
</script>

<button
    class={{
        'relative card card-hover aspect-square items-center w-full flex flex-col p-2 border-[1px] border-transparent hover:border-primary-100-900 overflow-hidden': true,
        'preset-filled-surface-100-900 !bg-opacity-50': !coverImage
    }}
    onclick={createPlaylist}
    oncontextmenu={handleContextMenu}
>
    {#if coverImage}
        <img
            src={coverImage}
            alt={`${track.album} cover art`}
            class="absolute left-0 top-0 w-full h-full"
        />
    {/if}
    <div class="relative">
        <h3
            class="type-scale-5 text-primary-800-200 line-clamp-3 cursor-pointer"
        >
            {track.title}
        </h3>
        <p class="opacity-60 cursor-pointer">{track.album}</p>
    </div>
</button>

{#if showContextMenu}
    <ContextMenu
        x={contextMenuX}
        y={contextMenuY}
        onclose={() => (showContextMenu = false)}
        items={[
            {
                Icon: Tag,
                label: 'Edit tags',
                onclick: async () => {
                    showTagEditor = !showTagEditor
                }
            },
            {
                Icon: Plus,
                label: 'Add to queue',
                onclick: async () => await addToQueue(QueueMethod.Priority)
            },
            {
                Icon: Replace,
                label: 'Replace current track',
                onclick: async () =>
                    await addToQueue(QueueMethod.OverwriteCurrent)
            },
            {
                Icon: Layers,
                label: 'Play overlayed',
                onclick: async () => await playParallel(false)
            },
            {
                Icon: Layers,
                label: 'Play overlayed (looping)',
                onclick: async () => await playParallel(true)
            }
        ]}
    />
{/if}

<TagEditor bind:open={showTagEditor} {track} />
