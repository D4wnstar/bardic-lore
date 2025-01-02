<script lang="ts">
    import { Search } from 'lucide-svelte'
    import SongBox from '$lib/SongBox.svelte'
    import CurrentlyPlaying from '$lib/CurrentlyPlaying.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { Track } from '$lib/types'
    import { TRACKS_FILENAME, TRACKS_SETTING } from '$lib/store'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'

    async function getTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        tracks = (await store.get<Track[]>(TRACKS_SETTING)) ?? []
    }

    let tracks: Track[] = $state([])
    onMount(getTracks)
</script>

<div class="grid grid-cols-[auto_1fr] h-screen">
    <LeftSidebar {getTracks} />
    <main class="flex flex-col p-4 min-h-0">
        <div class="flex grow min-h-0">
            <div class="flex grow flex-col">
                <header
                    class="bg-surface-100-900 mx-auto mb-4 flex h-12 w-1/2 min-w-[300px] max-w-[600px] items-center justify-center gap-2 rounded-md px-2"
                >
                    <Search />
                    <input
                        type="search"
                        class="h-12 grow border-none bg-transparent focus:ring-0"
                        placeholder="Search songs..."
                    />
                </header>
                <div class="mr-4 flex flex-wrap gap-2 overflow-y-auto p-1">
                    {#each tracks as track}
                        <SongBox title={track.title} tags={track.album} />
                    {:else}
                        <div class="type-scale-5">No songs!</div>
                    {/each}
                </div>
            </div>
            <CurrentlyPlaying />
        </div>

        <PlayerBar />
    </main>
</div>
