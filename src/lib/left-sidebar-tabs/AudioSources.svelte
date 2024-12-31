<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { type AudioSource } from '$lib/types'
    import SourceBox from './SourceBox.svelte'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { AUDIO_SOURCES_SETTING, SETTINGS_FILENAME } from '$lib/store'

    interface Props {
        getTracks: Function
    }
    let { getTracks }: Props = $props()

    let sources: AudioSource[] = $state([])
    const toast: ToastContext = getContext('toast')

    async function selectFolder() {
        let audioSources = await invoke<AudioSource[]>(
            'add_audio_sources'
        ).catch((reason) => {
            console.error(reason)
            toast.create({
                title: 'Error',
                description: reason,
                type: 'error'
            })
        })
        if (audioSources) {
            audioSources.forEach((s) => {
                if (!sources.find((s1) => s1.path == s.path)) {
                    sources.push(s)
                }
            })
        }
    }

    async function refreshFiles() {
        await invoke('refresh_audio_files').catch((err) => {
            console.error(`Error while getting audio sources. Error: ${err}`)
        })
        await getTracks()
    }

    onMount(async () => {
        const store = await load(SETTINGS_FILENAME, { autoSave: false })
        sources = (await store.get<AudioSource[]>(AUDIO_SOURCES_SETTING)) ?? []
    })
</script>

<div id="sources-sidebar" class="flex flex-col space-y-2">
    <h3 class="h3 text-center">Audio Sources</h3>
    <small class="small px-2 text-center">
        Audio Sources are the folders in which your audio files are contained.
        Audio files in these folders (and optionally sub-folders) will appear in
        the menu.
    </small>
    {#each sources as source}
        <SourceBox {source} />
    {/each}

    <div class="mx-2 flex justify-center gap-x-2">
        <button
            class="preset-outlined-primary-500 btn self-center"
            onclick={selectFolder}
        >
            Add sources
        </button>
        <button
            class="preset-outlined-primary-500 btn self-center"
            onclick={refreshFiles}
        >
            Refresh files
        </button>
    </div>
</div>
