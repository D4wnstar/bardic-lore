<script lang="ts">
    import SongBox from '$lib/SongBox.svelte'
    import SearchBar from '$lib/SearchBar.svelte'
    import { Folder, Wind } from 'lucide-svelte'
    import { rgbToHex } from '$lib/utils/utils'
    import { fade } from 'svelte/transition'
    import VirtualList from '$lib/utils/VirtualList.svelte'
    import { appTracks } from '$lib/stores.svelte'

    let iconColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-500')
    )
</script>

<div class="flex grow flex-col min-h-0">
    <SearchBar />
    {#if appTracks.size > 0}
        <VirtualList
            items={appTracks.visible().toSorted((a, b) => {
                const titleA = a.title ?? a.filename
                const titleB = b.title ?? b.filename
                return titleA.localeCompare(titleB)
            })}
        >
            {#snippet children(track)}
                {#key track}
                    <div transition:fade={{ duration: 100 }}>
                        <SongBox {track} />
                    </div>
                {/key}
            {/snippet}
        </VirtualList>
    {:else}
        <div
            class="type-scale-6 text-surface-800-200 text-center flex flex-col gap-2 justify-center items-center h-full"
        >
            <p>It's a little empty here...</p>
            <p>
                Use the <Folder class="inline mx-1" /> sidebar on the left to add
                some music!
            </p>
            <Wind size="144" color={iconColor} />
        </div>
    {/if}
</div>
