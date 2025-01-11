<script lang="ts">
    import { Tabs } from '@skeletonlabs/skeleton-svelte'
    import QueuedTrack from './QueuedTrack.svelte'
    import { appState } from '$lib/stores.svelte'

    let tabState = $state('queue')
    let queued = $derived(appState.playlist.queued().queued)
    let priority = $derived(appState.playlist.queued().priority)
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
                {#if queued.length > 0}
                    <p class="pl-1 type-scale-3">
                        <strong>You are listening to</strong>
                    </p>
                    <QueuedTrack track={queued[0]} />
                {/if}

                {#each priority as track, idx}
                    {#if idx === 0}
                        <p class="pl-1 type-scale-3 pt-4">
                            <strong>Up next</strong>
                        </p>
                    {/if}
                    <QueuedTrack {track} />
                {/each}

                {#each queued.slice(1) as track, idx}
                    {#if idx === 0}
                        <p class="pl-1 type-scale-3 pt-4">
                            <strong>Up next from the playlist</strong>
                        </p>
                    {/if}

                    <QueuedTrack {track} />
                {/each}
            </Tabs.Panel>
            <Tabs.Panel value="recent" classes="space-y-2 pb-5">
                {#each appState.recentlyPlayed as track}
                    <QueuedTrack {track} />
                {/each}
            </Tabs.Panel>
        {/snippet}
    </Tabs>
</div>
