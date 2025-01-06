<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { type AudioSource } from '$lib/types'
    import SourceBox from './SourceBox.svelte'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import {
        AUDIO_SOURCES_SETTING,
        SETTINGS_FILENAME
    } from '$lib/stores.svelte'

    interface Props {
        getTracks: Function
    }
    let { getTracks }: Props = $props()

    let sources: AudioSource[] = $state([])
    const toast: ToastContext = getContext('toast')

    async function getAudioSources() {
        const store = await load(SETTINGS_FILENAME, { autoSave: false })
        sources = (await store.get<AudioSource[]>(AUDIO_SOURCES_SETTING)) ?? []
    }

    async function selectFolder() {
        await invoke<AudioSource[]>('add_audio_sources').catch((err) => {
            console.error(err)
            toast.create({
                title: 'Error',
                description: err,
                type: 'error'
            })
        })
        await getAudioSources()
        await refreshTracks()
    }

    async function refreshTracks(response = false) {
        await invoke('refresh_audio_files')
            .then(() => {
                if (response) {
                    toast.create({
                        title: '',
                        description: 'Refreshed files',
                        type: 'info'
                    })
                }
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
        await getTracks()
    }

    onMount(getAudioSources)
</script>

<div id="sources-sidebar" class="flex flex-col h-full min-h-0 pb-5 space-y-2">
    <h3 class="h3 text-center">Audio Sources</h3>
    <small class="small px-3 text-justify">
        Audio Sources are the folders in which your audio files are contained.
        Audio files in these folders (and optionally sub-folders) will appear in
        the menu.
    </small>

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
            onclick={async () => await refreshTracks(true)}
        >
            Refresh files
        </button>
    </div>
</div>
