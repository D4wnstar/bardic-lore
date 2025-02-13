<script lang="ts">
    import '../app.css'
    import { getContext, onDestroy, onMount, type Snippet } from 'svelte'
    import {
        ToastProvider,
        type ToastContext
    } from '@skeletonlabs/skeleton-svelte'
    import LeftSidebar from '$lib/LeftSidebar.svelte'
    import RightSidebar from '$lib/RightSidebar.svelte'
    import type { CachedTrack, Tag } from '$lib/types'
    import { TagSet } from '$lib/state/tagset.svelte'
    import {
        appTags,
        appTracks,
        settings,
        TRACKS_FILENAME,
        TRACKS_SETTING
    } from '$lib/stores.svelte'
    import { listen, type UnlistenFn } from '@tauri-apps/api/event'
    import { BOT_ERROR, CLIENT_CONNECTED } from '$lib/events'
    import { createDiscordClient } from '$lib/utils/utils'
    import { load } from '@tauri-apps/plugin-store'
    import { TagGroupSet } from '$lib/state/taggroupset.svelte'
    import { TrackSet } from '$lib/state/trackset.svelte'
    import PlayerBar from '$lib/PlayerBar.svelte'

    let { children }: { children: Snippet } = $props()

    // function addTrack(track: CachedTrack) {
    //     const exists = appTracks.find((mt) => mt.track.path === track.path)
    //     if (!exists) {
    //         const sortedIndex = appTracks.findIndex((mt) => {
    //             const titleA = mt.track.title ?? mt.track.filename
    //             const titleB = track.title ?? track.filename
    //             return titleA.localeCompare(titleB) === 1
    //         })
    //         appTracks.splice(sortedIndex, 0, { track, visible: true })
    //     }
    // }

    // function removeTrack(track: CachedTrack) {
    //     const idxToDelete = appTracks.findIndex(
    //         (mt) => mt.track.path === track.path
    //     )
    //     if (idxToDelete >= 0) appTracks.splice(idxToDelete, 1)
    // }

    async function getCachedTracks() {
        const store = await load(TRACKS_FILENAME, { autoSave: false })
        const cachedTracks =
            (await store.get<CachedTrack[]>(TRACKS_SETTING)) ?? []
        //@ts-expect-error
        cachedTracks.forEach((t) => (t.tags = new TagSet(t.tags)))
        appTracks.clear()
        appTracks.union(new TrackSet(cachedTracks))

        // Reset album and artist groups to refresh tags based on available appTracks
        // appTags.delete(TagGroupSet.ALBUM_GROUP)
        // appTags.add({
        //     name: TagGroupSet.ALBUM_GROUP,
        //     tagSet: new TagSet([]),
        //     builtin: true,
        //     modifiable: false
        // })
        // appTags.delete(TagGroupSet.ARTIST_GROUP)
        // appTags.add({
        //     name: TagGroupSet.ARTIST_GROUP,
        //     tagSet: new TagSet([]),
        //     builtin: true,
        //     modifiable: false
        // })

        // const cachedSet = new TrackSet(
        //     cachedTracks
        //         .map((track) => {
        //             // TODO: This function makes refreshing appTracks laggy: improve this
        //             if (settings.hideOst && track.album) {
        //                 track.album = track.album
        //                     .replace(
        //                         /:? *\(?Complete[^:()]*?Soundtrack\)?/i,
        //                         ''
        //                     )
        //                     .replace(/:? *\(?Deluxe[^:()]*?Soundtrack\)?/i, '')
        //                     .replace(/:? *\(?Original[^:()]*?Score\)?/i, '')
        //                     .replace(
        //                         /:? *\(?Original[^:()]*?Soundtrack\)?/i,
        //                         ''
        //                     )
        //             }

        //             if (settings.showAlbumTags && track.album) {
        //                 const newTag: Tag = {
        //                     value: track.album,
        //                     owners: new TrackSet([track]),
        //                     group: TagGroupSet.ALBUM_GROUP
        //                 }
        //                 if (appTags.getTag(track.album)) {
        //                     appTags.addTagOwners(
        //                         TagGroupSet.ALBUM_GROUP,
        //                         newTag
        //                     )
        //                 } else {
        //                     appTags.addTag(TagGroupSet.ALBUM_GROUP, newTag)
        //                 }
        //             }

        //             if (settings.showArtistTags && track.artist) {
        //                 const newTag: Tag = {
        //                     value: track.artist,
        //                     owners: new TrackSet([track]),
        //                     group: TagGroupSet.ARTIST_GROUP
        //                 }
        //                 if (appTags.getTag(track.artist)) {
        //                     appTags.addTagOwners(
        //                         TagGroupSet.ARTIST_GROUP,
        //                         newTag
        //                     )
        //                 } else {
        //                     appTags.addTag(TagGroupSet.ARTIST_GROUP, newTag)
        //                 }
        //             }

        //             return { ...track, visible: true }
        //         })
        //         .toSorted((a, b) => {
        //             const titleA = a.title ?? a.filename
        //             const titleB = b.title ?? b.filename
        //             return titleA.localeCompare(titleB)
        //         })
        // )

        // // Clear tracks first
        // appTracks.clear()
        // appTracks.union(cachedSet)
    }

    let unlisten: UnlistenFn[] = []
    onMount(async () => {
        const toast: ToastContext = getContext('toast')
        await getCachedTracks()

        const unlisten1 = await listen<string>(BOT_ERROR, (ev) => {
            toast.create({
                title: 'Error',
                description: ev.payload,
                type: 'error'
            })
        })

        const unlisten2 = await listen<undefined>(CLIENT_CONNECTED, () => {
            toast.create({
                title: 'Created client',
                description:
                    'Successfully created client. Servers should refresh in a moment.',
                type: 'success'
            })
        })
        unlisten.push(unlisten1)
        unlisten.push(unlisten2)

        if (settings.autoconnect) {
            await createDiscordClient(toast)
        }
    })

    onDestroy(() => {
        unlisten.forEach((fn) => fn())
    })
</script>

<svelte:head>
    <title>Bardic Lore</title>
</svelte:head>

<ToastProvider>
    <!-- Prevent drag-to-select on the whole UI (except modals) -->
    <div class="h-screen w-screen select-none">
        <div class="flex h-screen">
            <LeftSidebar {getCachedTracks} />
            <main class="flex flex-col p-4 min-h-0 grow">
                {@render children()}
                <PlayerBar />
            </main>
            <RightSidebar />
        </div>
    </div>
</ToastProvider>
