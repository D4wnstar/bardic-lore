<script lang="ts">
    import { Search } from 'lucide-svelte'
    import SongBox from '$lib/SongBox.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import type { Track } from '$lib/types'
    import {
        playerState,
        trackQueue,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { getContext, onDestroy, onMount } from 'svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { type UnlistenFn, listen } from '@tauri-apps/api/event'
    import { BOT_ERROR, TRACK_ENDED, UPDATE_TRACK } from '$lib/events'

    let tracks: Track[] = $state([])

    async function getTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        tracks = (await store.get<Track[]>(TRACKS_SETTING)) ?? []
    }

    let unlisten: UnlistenFn[] = []
    onMount(async () => {
        getTracks()
        const toast: ToastContext = getContext('toast')

        // Setup all the global event listeners
        let unlisten1 = await listen<string>(BOT_ERROR, (ev) => {
            toast.create({
                title: 'Error',
                description: ev.payload,
                type: 'error'
            })
        })
        unlisten.push(unlisten1)

        let unlisten2 = await listen<any>(UPDATE_TRACK, (ev) => {
            playerState.playing = ev.payload['playing'] ?? playerState.playing
        })
        unlisten.push(unlisten2)

        let unlisten3 = await listen<any>(TRACK_ENDED, (ev) => {
            trackQueue.tracks.shift()
            playerState.playing = false
            playerState.trackProgress = 0
        })
        unlisten.push(unlisten3)
    })

    onDestroy(() => {
        for (const unlistenFn of unlisten) {
            unlistenFn()
        }
    })
</script>

<div class="flex h-screen">
    <LeftSidebar {getTracks} />
    <main class="flex flex-col p-4 min-h-0 grow">
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
                        <SongBox {track} />
                    {:else}
                        <div class="type-scale-5">No songs!</div>
                    {/each}
                </div>
            </div>
        </div>

        <PlayerBar />
    </main>
    <RightSidebar />
</div>
