<script>
    import { appState } from '$lib/stores.svelte'
    import ParallelTrack from './ParallelTrack.svelte'
</script>

<div class="space-y-2 px-2">
    <div class="pl-1"><strong>Currently playing</strong></div>
    {#if appState.trackQueue[0]}
        <ParallelTrack
            main
            state={{
                track: appState.trackQueue[0],
                player: appState.mainPlayer
            }}
        />
    {/if}
    {#each appState.parallelTracks as state}
        <ParallelTrack {state} />
    {/each}
</div>
