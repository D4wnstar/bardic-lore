<script lang="ts">
    import { recentlyPlayed, trackQueue } from '$lib/stores.svelte'
    import { Tabs } from '@skeletonlabs/skeleton-svelte'
    import QueuedTrack from './QueuedTrack.svelte'

    let tabState = $state('queue')
</script>

<div class="px-3 overflow-auto">
    <Tabs bind:value={tabState} listJustify="justify-center">
        {#snippet list()}
            <Tabs.Control
                value="queue"
                labelBase="btn hover:preset-filled-primary-500 rounded-none"
                >Queue</Tabs.Control
            >
            <Tabs.Control
                value="recent"
                labelBase="btn hover:preset-filled-primary-500 rounded-none"
                >Recently played</Tabs.Control
            >
        {/snippet}
        {#snippet content()}
            <Tabs.Panel value="queue" classes="space-y-2 pb-5">
                {#each trackQueue.tracks as track, idx}
                    {#if idx === 0}
                        <p class="pl-1 type-scale-3">
                            <strong>You are listening to</strong>
                        </p>
                    {:else if idx === 1}
                        <p class="pl-1 type-scale-3 pt-4">
                            <strong>Up next</strong>
                        </p>
                    {/if}
                    <QueuedTrack {track} />
                {/each}
            </Tabs.Panel>
            <Tabs.Panel value="recent" classes="space-y-2 pb-5">
                {#each recentlyPlayed.tracks as track}
                    <QueuedTrack {track} />
                {/each}
            </Tabs.Panel>
        {/snippet}
    </Tabs>
</div>
