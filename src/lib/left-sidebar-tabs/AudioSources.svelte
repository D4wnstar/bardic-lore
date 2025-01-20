<script lang="ts">
    import { Channel, invoke } from '@tauri-apps/api/core'
    import { type AudioSource, type CachedTrack } from '$lib/types'
    import SourceBox from './SourceBox.svelte'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import {
        AUDIO_SOURCES_SETTING,
        SETTINGS_FILENAME
    } from '$lib/stores.svelte'

    interface Props {
        addTrack: (track: CachedTrack) => void
        removeTrack: (track: CachedTrack) => void
        getCachedTracks: () => Promise<void>
    }
    let { addTrack, removeTrack, getCachedTracks }: Props = $props()

    let sources: AudioSource[] = $state([])
    const toast: ToastContext = getContext('toast')

    async function getAudioSources() {
        const store = await load(SETTINGS_FILENAME, { autoSave: false })
        sources = (await store.get<AudioSource[]>(AUDIO_SOURCES_SETTING)) ?? []
    }

    async function selectFolder() {
        const newSources = await invoke<AudioSource[]>(
            'add_audio_sources'
        ).catch((err) => {
            console.error(err)
            toast.create({
                title: 'Error',
                description: err,
                type: 'error'
            })
        })

        if (newSources) {
            await getAudioSources()
            await refreshTracks(newSources)
        }
    }

    type TrackPacket =
        | {
              event: 'add'
              track: CachedTrack
          }
        | {
              event: 'remove'
              track: CachedTrack
          }
        | { event: 'refresh' }

    async function refreshTracks(sources?: AudioSource[], reset?: boolean) {
        toast.create({
            title: '',
            description:
                'Refreshing files. This may take a while, especially for the first time if the tracks have cover images.',
            type: 'info'
        })

        const onGetTrack = new Channel<TrackPacket>()
        onGetTrack.onmessage = (packet) => {
            if (packet.event === 'add') {
                addTrack(packet.track)
            } else if (packet.event === 'remove') {
                removeTrack(packet.track)
            } else if (packet.event === 'refresh') {
                getCachedTracks()
            }
        }

        await invoke('update_tracks_from_sources', {
            sources,
            reset,
            onGetTrack
        })
            .then(() => {
                toast.create({
                    title: '',
                    description: 'Refreshed files.',
                    type: 'info'
                })
            })
            .catch((err) => {
                console.error(
                    `Error while getting audio sources. Error: ${err}`
                )
                toast.create({
                    title: 'Error',
                    description: err,
                    type: 'error'
                })
            })
    }

    onMount(getAudioSources)
</script>

<div id="sources-sidebar" class="flex flex-col h-full min-h-0 pb-5 space-y-2">
    <h3 class="type-scale-7 heading-font-weight px-2 text-primary-900-100">
        Audio Sources
    </h3>
    <p class="type-scale-1 px-2 text-justify">
        Audio Sources are the folders in which your audio files are contained.
        Audio files in these folders (and optionally sub-folders) will appear in
        the menu.
    </p>

    <div class="overflow-auto space-y-2">
        {#each sources as source}
            <SourceBox {source} {refreshTracks} {getAudioSources} />
        {/each}
    </div>

    <div class="flex justify-center gap-x-2 pt-2">
        <button
            class="preset-outlined-primary-400-600 btn self-center"
            onclick={selectFolder}
        >
            Add sources
        </button>
        <button
            class="preset-outlined-primary-400-600 btn self-center"
            onclick={async () => {
                await getAudioSources()
                await refreshTracks(undefined, true)
            }}
        >
            Refresh files
        </button>
    </div>
</div>
