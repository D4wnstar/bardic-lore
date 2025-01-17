<script lang="ts">
    import { appState } from '$lib/stores.svelte'
    import type { Track } from '$lib/types'
    import { fade } from 'svelte/transition'
    import TrackControls from './TrackControls.svelte'
</script>

<div class="space-y-2 px-2 pb-5">
    <div class="pl-1"><strong>Currently playing</strong></div>
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
