<script lang="ts">
    import { trackQueue } from '$lib/stores.svelte'
    import QueuedTrack from './QueuedTrack.svelte'
</script>

<div class="space-y-2 px-3">
    {#each trackQueue.tracks as track, idx}
        {#if idx === 0}
            <p class="type-scale-3"><strong>You are listening to</strong></p>
        {:else if idx === 1}
            <p class="type-scale-3 pt-4"><strong>Up next</strong></p>
        {/if}
        <QueuedTrack {track} />
    {/each}
</div>
