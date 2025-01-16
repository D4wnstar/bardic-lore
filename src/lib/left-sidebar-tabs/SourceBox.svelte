<script lang="ts">
    import type { AudioSource } from '$lib/types'
    import { basename } from '@tauri-apps/api/path'
    import { getContext } from 'svelte'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { Trash2 } from 'lucide-svelte'
    import ZaglessSwitch from '$lib/utils/ZaglessSwitch.svelte'

    interface Props {
        source: AudioSource
        refreshTracks: Function
        getAudioSources: Function
    }

    let { source, refreshTracks, getAudioSources }: Props = $props()

    const toast: ToastContext = getContext('toast')

    async function updateAudioSource(
        oldPath: string,
        path: string,
        active: boolean,
        recursive: boolean
    ) {
        await invoke<AudioSource[]>('update_audio_source', {
            oldPath: oldPath,
            path: path,
            active: active,
            recursive: recursive
        }).catch((reason) => {
            console.error(reason)
            toast.create({
                title: 'Error',
                description: reason,
                type: 'error'
            })
        })

        await refreshTracks()
    }

    async function deleteAudioSource(path: string) {
        await invoke<AudioSource[]>('delete_audio_source', {
            path: path
        }).catch((reason) => {
            console.error(reason)
            toast.create({
                title: 'Error',
                description: reason,
                type: 'error'
            })
        })

        await getAudioSources()
        await refreshTracks()
    }

    let namePromise = $derived(basename(source.path))
    let active = $state(source.active)
    let recursive = $state(source.recursive)
</script>

<div
    class={`preset-filled-surface-100-900 !bg-opacity-50 mx-2 rounded-md border-[1px] border-primary-100-900 !border-opacity-70 px-4 py-2 ${active ? '' : 'opacity-50'}`}
>
    <div class="flex mb-2">
        <h4 class="type-scale-5 grow text-secondary-700-300">
            <strong>{#await namePromise then name}{name}{/await}</strong>
        </h4>
        <button
            class="btn-icon rounded hover:preset-filled-surface-100-900"
            onclick={() => deleteAudioSource(source.path)}
        >
            <Trash2 />
        </button>
    </div>
    <div class="grid grid-cols-[1fr_auto] gap-y-2">
        <p>Active</p>
        <ZaglessSwitch
            name="active"
            bind:checked={active}
            onCheckedChange={(_state) =>
                updateAudioSource(source.path, source.path, active, recursive)}
        />
        <p>Include subfolders</p>
        <ZaglessSwitch
            name="recursive"
            bind:checked={recursive}
            onCheckedChange={(_state) => {
                updateAudioSource(source.path, source.path, active, recursive)
            }}
        />
    </div>
</div>
