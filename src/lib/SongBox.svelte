<script lang="ts">
    import {
        globalGuild,
        playerState,
        toOverwrite,
        trackQueue
    } from './stores.svelte'
    import type { Track } from './types'
    import { emit } from '@tauri-apps/api/event'
    import { QUEUE_TRACK } from '$lib/events'
    import ContextMenu from './ContextMenu.svelte'
    import { Plus } from 'lucide-svelte'

    interface Props {
        track: Track
    }

    let { track }: Props = $props()

    let showContextMenu = $state(false)
    let contextMenuX = $state(0)
    let contextMenuY = $state(0)

    $inspect(showContextMenu)

    function handleContextMenu(event: MouseEvent) {
        event.preventDefault()
        contextMenuX = event.clientX
        contextMenuY = event.clientY
        showContextMenu = true
    }

    async function addToQueue(overwrite: boolean) {
        if (!playerState.offline) {
            if (overwrite && trackQueue.tracks.length > 0) {
                toOverwrite.track = track
            } else {
                trackQueue.tracks.push(track)
            }
            await emit(QUEUE_TRACK, {
                guildId: globalGuild.id,
                trackData: track,
                looping: playerState.looping,
                overwrite,
                prepend: false,
                volume: playerState.volume
            })
        }
    }
</script>

<button
    class="card card-hover preset-filled-surface-100-900 !bg-opacity-50 flex flex-[10rem] flex-col items-center space-y-2 p-2 text-center border-[1px] border-transparent hover:border-primary-100-900"
    onclick={async () => await addToQueue(true)}
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
                onclick: async () => await addToQueue(false)
            }
        ]}
    />
{/if}
