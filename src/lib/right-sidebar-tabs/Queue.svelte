<script lang="ts">
    import { Tabs } from '@skeletonlabs/skeleton-svelte'
    import QueuedTrack from './QueuedTrack.svelte'
    import { appState } from '$lib/stores.svelte'
    import { fade, slide } from 'svelte/transition'
    import { flip } from 'svelte/animate'
    import { AudioLines } from 'lucide-svelte'
    import { rgbToHex } from '$lib/utils/utils'

    let tabState = $state('queue')
    let queued = $derived(appState.playlist.queue)
    let priority = $derived(appState.playlist.priority)

    const maxTracksShown = 20

    const iconColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-500')
    )
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
                    <p
                        class="pl-1 type-scale-3"
                        transition:fade={{ duration: 200 }}
                    >
                        <b>You are listening to</b>
                    </p>
                    {#key queued[0]}
                        <div in:fade={{ duration: 200 }}>
                            <QueuedTrack track={queued[0]} />
                        </div>
                    {/key}
                {:else if appState.playlist.isEmpty()}
                    <div
                        class="flex flex-col gap-2"
                        in:fade={{ delay: 200, duration: 500 }}
                    >
                        <p class="text-surface-800-200 text-center">
                            This where your songs will be once you play
                            something.
                        </p>
                        <AudioLines
                            size="96"
                            color={iconColor}
                            class="self-center"
                        />
                    </div>
                {/if}

                {#if priority.length > 0}
                    <p
                        class="pl-1 type-scale-3 pt-4"
                        transition:fade={{ duration: 200 }}
                    >
                        <b>Up next</b>
                    </p>
                {/if}
                {#each priority as track (track)}
                    <div transition:slide={{ axis: 'y' }}>
                        <QueuedTrack {track} />
                    </div>
                {/each}

                {#if queued.slice(1).length > 0}
                    <p
                        class="pl-1 type-scale-3 pt-4"
                        transition:fade={{ duration: 200 }}
                    >
                        <b>Up next from the playlist</b>
                    </p>
                {/if}
                {#each queued.slice(1, maxTracksShown) as track (track)}
                    <div transition:slide={{ axis: 'y' }}>
                        <QueuedTrack {track} />
                    </div>
                {/each}
                {#if queued.length > maxTracksShown}
                    <p
                        class="pl-1 type-scale-3 text-center"
                        transition:fade={{ duration: 200 }}
                    >
                        and {queued.length - maxTracksShown} more...
                    </p>
                {/if}
            </Tabs.Panel>
            <Tabs.Panel value="recent" classes="space-y-2 pb-5">
                {#each appState.recentlyPlayed
                    .slice(0, maxTracksShown)
                    .toReversed() as track (track)}
                    <div animate:flip={{ duration: 300 }}>
                        <QueuedTrack {track} />
                    </div>
                {:else}
                    <div class="flex flex-col gap-2">
                        <p class="text-surface-800-200 text-center">
                            This where your songs will be once you play
                            something.
                        </p>
                        <AudioLines
                            size="96"
                            color={iconColor}
                            class="self-center"
                        />
                    </div>
                {/each}
            </Tabs.Panel>
        {/snippet}
    </Tabs>
</div>
