<script lang="ts">
    import { appState } from '$lib/stores.svelte'
    import type { Track } from '$lib/types'
    import TrackControls from './TrackControls.svelte'
</script>

<div class="space-y-2 px-2 pb-5">
    <div class="pl-1"><strong>Currently playing</strong></div>
    {#if appState.playlist.current()}
        <TrackControls
            state={{
                track: appState.playlist.current() as Track,
                player: appState.player
            }}
        />
    {/if}
    {#if appState.parallelPlayers.length > 0}
        <div class="pl-1"><strong>Overlayed</strong></div>
    {/if}
    {#each appState.parallelPlayers as state}
        <TrackControls {state} parallel />
    {/each}
</div>
