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
        refreshTracks: (sources?: AudioSource[]) => Promise<void>
        getAudioSources: () => Promise<void>
    }

    let { source, refreshTracks, getAudioSources }: Props = $props()

    const toast: ToastContext = getContext('toast')

    async function updateAudioSource() {
        const wasUpdated = await invoke<boolean>('update_audio_source', {
            source
        }).catch((reason) => {
            console.error(reason)
            toast.create({
                title: 'Error',
                description: reason,
                type: 'error'
            })
        })

        if (wasUpdated) {
            await refreshTracks([source])
        }
    }

    async function deleteAudioSource() {
        const wasDeleted = await invoke<AudioSource[]>('delete_audio_source', {
            source
        }).catch((reason) => {
            console.error(reason)
            toast.create({
                title: 'Error',
                description: reason,
                type: 'error'
            })
        })

        if (wasDeleted) {
            await refreshTracks([{ ...source, active: false }])
            await getAudioSources()
        }
    }

    let namePromise = $derived(basename(source.path))
</script>

<div
    class={{
        'preset-filled-surface-100-900 !bg-opacity-50 mx-2 rounded-md border-[1px] border-primary-100-900 !border-opacity-70 px-4 py-2': true,
        'opacity-50': !source.active
    }}
>
    <div class="flex mb-2">
        <h4 class="type-scale-5 grow text-secondary-700-300">
            <strong>{#await namePromise then name}{name}{/await}</strong>
        </h4>
        <button
            class="btn-icon rounded hover:preset-filled-surface-100-900"
            onclick={deleteAudioSource}
        >
            <Trash2 />
        </button>
    </div>
    <div class="grid grid-cols-[1fr_auto] gap-y-2">
        <p>Active</p>
        <ZaglessSwitch
            name="active"
            bind:checked={source.active}
            onCheckedChange={updateAudioSource}
        />
        <p>Include subfolders</p>
        <ZaglessSwitch
            name="recursive"
            bind:checked={source.recursive}
            onCheckedChange={updateAudioSource}
        />
    </div>
</div>
