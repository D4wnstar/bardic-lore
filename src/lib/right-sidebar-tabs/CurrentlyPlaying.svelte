<script lang="ts">
    import { appState } from '$lib/stores.svelte'
    import type { Track } from '$lib/types'
    import { fade } from 'svelte/transition'
    import TrackControls from './TrackControls.svelte'
    import { AudioLines } from 'lucide-svelte'
    import { rgbToHex } from '$lib/utils/utils'

    let iconColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-500')
    )
</script>

<div class="space-y-2 px-2 pb-5">
    <p class="pl-1"><b>Currently playing</b></p>
    {#if appState.playlist.isEmpty()}
        <div class="flex flex-col gap-2">
            <p class="text-surface-800-200 text-center">
                This where your songs will be once you play something.
            </p>
            <AudioLines size="96" color={iconColor} class="self-center" />
        </div>
    {/if}
    {#if appState.playlist.current()}
        <div transition:fade={{ duration: 100 }}>
            <TrackControls
                parallelState={{
                    track: appState.playlist.current() as Track,
                    player: appState.player
                }}
            />
        </div>
    {/if}
    {#if appState.parallel.states.length > 0}
        <div class="pl-1"><strong>Overlayed</strong></div>
    {/if}
    {#each appState.parallel.states as state}
        <div transition:fade={{ duration: 100 }}>
            <TrackControls parallelState={state} parallel />
        </div>
    {/each}
</div>
