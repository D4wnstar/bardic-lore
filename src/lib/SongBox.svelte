<script lang="ts">
    import { appState, skipRemoveOnEnd } from './stores.svelte'
    import type { CachedTrack } from './types'
    import { emit } from '@tauri-apps/api/event'
    import {
        PLAY_PARALLEL,
        QUEUE_TRACK,
        QueueMethod,
        type PlayParallelPayload,
        type QueueTrackPayload,
        CREATE_PLAYLIST,
        type CreatePlaylistPayload
    } from '$lib/events'
    import ContextMenu from './ContextMenu.svelte'
    import { Layers, Plus, Replace } from 'lucide-svelte'
    import { LoopState } from './state.svelte'
    import { getCover, permuteTracks } from './utils/utils'
    import { onMount } from 'svelte'

    interface Props {
        track: CachedTrack
    }

    let { track }: Props = $props()

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
            appState.availableTracks
        ) as CachedTrack[]
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

    onMount(async () => {
        coverImage = await getCover(track.cover_path, track.cover_filetype)
    })
</script>

<button
    class={{
        'card card-hover aspect-square flex flex-[10rem] xl:flex-[12rem] max-w-[14rem] flex-col items-center space-y-2 p-2 text-center border-[1px] border-transparent hover:border-primary-100-900': true,
        'relative overflow-hidden': coverImage,
        'preset-filled-surface-100-900 !bg-opacity-50': !coverImage
    }}
    onclick={createPlaylist}
    oncontextmenu={handleContextMenu}
>
    {#if coverImage}
        <img
            src={coverImage}
            alt="Album cover"
            class="opacity-30 absolute left-0 top-0 h-full w-full brightness-50"
        />
    {/if}
    <div class="relative">
        <h3 class="type-scale-5 text-primary-800-200 line-clamp-3">
            {track.title}
        </h3>
        <p class="opacity-60">{track.album}</p>
    </div>
</button>

{#if showContextMenu}
    <ContextMenu
        x={contextMenuX}
        y={contextMenuY}
        onclose={() => (showContextMenu = false)}
        items={[
            {
                Icon: Plus,
                label: 'Add to queue',
                onclick: async () => await addToQueue(QueueMethod.Priority)
            },
            {
                Icon: Replace,
                label: 'Replace current',
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
