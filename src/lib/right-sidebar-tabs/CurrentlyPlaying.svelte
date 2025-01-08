<script>
    import { playerState } from '$lib/stores.svelte'
    import ParallelTrack from './ParallelTrack.svelte'
</script>

<div class="overflow-auto space-y-2 px-2">
    <div class="pl-1"><strong>Currently playing</strong></div>
    {#if playerState.trackQueue[0]}
        <ParallelTrack track={playerState.trackQueue[0]} />
    {/if}
    {#each playerState.parallelTracks as track}
        <ParallelTrack {track} />
    {/each}
</div>
