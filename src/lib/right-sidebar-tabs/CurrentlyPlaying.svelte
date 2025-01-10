<script>
    import { appState } from '$lib/stores.svelte'
    import TrackControls from './TrackControls.svelte'
</script>

<div class="space-y-2 px-2 pb-5">
    <div class="pl-1"><strong>Currently playing</strong></div>
    {#if appState.trackQueue[0]}
        <TrackControls
            state={{
                track: appState.trackQueue[0],
                player: appState.mainPlayer
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
