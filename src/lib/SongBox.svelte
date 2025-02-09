<script lang="ts">
    import {
        appState,
        appTags,
        settings,
        skipRemoveOnEnd,
        virtualListTop
    } from './stores.svelte'
    import {
        ALBUM_GROUP,
        ARTIST_GROUP,
        type CachedTrack,
        type MaskedTrack,
        type Tag
    } from './types'
    import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
    import {
        PLAY_PARALLEL,
        QUEUE_TRACK,
        QueueMethod,
        type PlayParallelPayload,
        type QueueTrackPayload,
        CREATE_PLAYLIST,
        type CreatePlaylistPayload
    } from '$lib/events'
    import ContextMenu from './popovers/ContextMenu.svelte'
    import { Layers, Plus, Replace, TagIcon } from 'lucide-svelte'
    import { getCover, permuteTracks } from './utils/utils'
    import { onDestroy, onMount } from 'svelte'
    import TagEditor from './popovers/TagEditor.svelte'
    import TagChip from './utils/TagChip.svelte'
    import { LoopState } from './state/player.svelte'

    interface Props {
        track: CachedTrack
        tracks: MaskedTrack[]
    }

    let { track, tracks }: Props = $props()

    let showTagEditor = $state(false)
    let showContextMenu = $state(false)
    let contextMenuX = $state(0)
    let contextMenuY = $state(0)
    let coverImage: string | undefined = $state()
    let tags = $state(appTags.getByTrack(track))

    function handleContextMenu(event: MouseEvent) {
        event.preventDefault()
        const target = event.currentTarget as HTMLElement
        // The VirtualList messes with the event positioning, so we
        // fix it by shifting the event coordinates by the bounding box
        // The y coordinate is also shifted by the virtual list offset
        const rect = target.getBoundingClientRect()
        contextMenuX = event.x - rect.width * 1.55
        contextMenuY = event.y - rect.height * 0.4 + virtualListTop.top
        // (i have no ideas why these exact multipliers work lol)
        showContextMenu = true
    }

    async function createPlaylist() {
        if (appState.offline) return
        // This is guaranteed to work because the track needs to be in the list
        // for us to even click on it
        const tracksToSend = permuteTracks(
            track,
            tracks.filter((mt) => mt.visible).map((mt) => mt.track)
        ) as CachedTrack[]
        await emit(CREATE_PLAYLIST, {
            guildId: appState.guildId,
            tracksData: tracksToSend,
            volume: appState.player.volume,
            loopFirst: appState.player.loopState === LoopState.LoopTrack,
            shuffle: appState.player.shuffle
        } satisfies CreatePlaylistPayload)
    }

    async function addToQueue(method: QueueMethod) {
        if (!appState.offline) {
            if (
                appState.playlist.current() &&
                method === QueueMethod.OverwriteCurrent
            ) {
                skipRemoveOnEnd.toSkip += 1
            }
            await emit(QUEUE_TRACK, {
                guildId: appState.guildId,
                trackData: track,
                looping: appState.player.loopState === LoopState.LoopTrack,
                queueMethod: method,
                volume: appState.player.volume
            } satisfies QueueTrackPayload)
        }
    }

    async function playParallel(looping: boolean) {
        if (!appState.offline) {
            await emit(PLAY_PARALLEL, {
                guildId: appState.guildId,
                trackData: track,
                volume: appState.player.volume,
                looping
            } satisfies PlayParallelPayload)
        }
    }

    function handleTagEdit(tag: Tag, outcome: 'add' | 'remove') {
        if (outcome === 'add') {
            tags.add(tag)
        } else if (outcome === 'remove') {
            tags.delete(tag)
        }
    }

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        const loadCover = async () => {
            if (settings.showCovers) {
                coverImage = await getCover('cover', track.coverHash)
            } else {
                coverImage = undefined
            }
        }

        await loadCover()
        unlisten = await listen('reload-cover', loadCover)
    })

    onDestroy(() => {
        if (unlisten) unlisten()
    })
</script>

<button
    class={{
        'relative card card-hover aspect-square items-center w-full flex flex-col p-2 border-[1px] border-transparent hover:border-primary-100-900 overflow-hidden': true,
        'preset-filled-surface-100-900 !bg-opacity-50': !coverImage
    }}
    onclick={createPlaylist}
    oncontextmenu={handleContextMenu}
>
    {#if coverImage}
        <img
            src={coverImage}
            alt={`${track.album} cover art`}
            class="absolute left-0 top-0 w-full h-full"
        />
    {/if}
    <div class="relative">
        <h3
            class="type-scale-5 text-primary-800-200 line-clamp-3 cursor-pointer"
        >
            {track.title}
        </h3>
        <p class="opacity-60 cursor-pointer line-clamp-3">{track.album}</p>
        <div class="mt-2 flex flex-wrap justify-center gap-1">
            {#each tags
                .sorted()
                .filter((tag) => tag.group !== ALBUM_GROUP && tag.group !== ARTIST_GROUP) as tag}
                <TagChip
                    {tag}
                    classes="hover:brightness-100"
                    preset="preset-tonal"
                />
            {/each}
        </div>
    </div>
</button>

{#if showContextMenu}
    <ContextMenu
        x={contextMenuX}
        y={contextMenuY}
        onclose={() => (showContextMenu = false)}
        items={[
            {
                Icon: TagIcon,
                label: 'Edit tags',
                onclick: async () => {
                    showTagEditor = !showTagEditor
                }
            },
            {
                Icon: Plus,
                label: 'Add to queue',
                onclick: async () => await addToQueue(QueueMethod.Priority)
            },
            {
                Icon: Replace,
                label: 'Replace current track',
                onclick: async () =>
                    await addToQueue(QueueMethod.OverwriteCurrent)
            },
            {
                Icon: Layers,
                label: 'Play overlayed',
                onclick: async () => await playParallel(false)
            },
            {
                Icon: Layers,
                label: 'Play overlayed (looping)',
                onclick: async () => await playParallel(true)
            }
        ]}
    />
{/if}

<TagEditor bind:open={showTagEditor} {track} {handleTagEdit} />
