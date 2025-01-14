<script lang="ts">
    import { appState, skipRemoveOnEnd } from './stores.svelte'
    import type { CachedTrack } from './types'
    import { emit } from '@tauri-apps/api/event'
    import {
        ADD_TRACK,
        PLAY_PARALLEL,
        QUEUE_TRACK,
        QueueMethod,
        type AddTrackPayload,
        type PlayParallelPayload,
        type QueueTrackPayload
    } from '$lib/events'
    import ContextMenu from './ContextMenu.svelte'
    import { Plus } from 'lucide-svelte'
    import { LoopState } from './state.svelte'

    interface Props {
        track: CachedTrack
        tracks: { track: CachedTrack; mask: boolean }[]
    }

    let { track, tracks }: Props = $props()

    let showContextMenu = $state(false)
    let contextMenuX = $state(0)
    let contextMenuY = $state(0)

    function handleContextMenu(event: MouseEvent) {
        event.preventDefault()
        contextMenuX = event.clientX
        contextMenuY = event.clientY
        showContextMenu = true
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
                volume: appState.player.volume,
                numberOfPriority: appState.playlist.priority.length,
                isPriorityPlaying: appState.playlist.isCurrentPriority()
            } satisfies QueueTrackPayload)

            // for (const otherTrack of tracks) {
            //     if (!otherTrack.mask || otherTrack.track === track) continue
            //     await emit(QUEUE_TRACK, {
            //         guildId: appState.guildId,
            //         trackData: otherTrack.track,
            //         looping: appState.player.loopState === LoopState.LoopTrack,
            //         queueMethod: method,
            //         volume: appState.player.volume,
            //         numberOfPriority: appState.playlist.priority.length,
            //         isPriorityPlaying: appState.playlist.isCurrentPriority()
            //     } satisfies QueueTrackPayload)
            // }
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
</script>

<button
    class="card card-hover preset-filled-surface-100-900 !bg-opacity-50 flex flex-[10rem] xl:flex-[12rem] max-w-[14rem] flex-col items-center space-y-2 p-2 text-center border-[1px] border-transparent hover:border-primary-100-900"
    onclick={async () => await addToQueue(QueueMethod.OverwriteCurrent)}
    oncontextmenu={handleContextMenu}
>
    <h3 class="type-scale-5 text-primary-800-200">
        {track.title}
    </h3>
    <p class="opacity-50">{track.album}</p>
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
                Icon: Plus,
                label: 'Play overlayed',
                onclick: async () => await playParallel(false)
            },
            {
                Icon: Plus,
                label: 'Play overlayed (looping)',
                onclick: async () => await playParallel(true)
            }
        ]}
    />
{/if}
