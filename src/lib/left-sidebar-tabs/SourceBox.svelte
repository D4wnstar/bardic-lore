<script lang="ts">
    import type { AudioSource } from '$lib/types'
    import { basename } from '@tauri-apps/api/path'
    import { onMount } from 'svelte'
    import { getContext } from 'svelte'
    import { type ToastContext, Switch } from '@skeletonlabs/skeleton-svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { Trash2 } from 'lucide-svelte'

    interface Props {
        source: AudioSource
        getTracks: Function
    }

    let { source, getTracks }: Props = $props()

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

        await getTracks()
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

        await getTracks()
    }

    let folderName = $state('')
    onMount(async () => (folderName = await basename(source.path)))

    let active = $state(source.active)
    let recursive = $state(source.recursive)
</script>

<div
    class={`variant-soft-secondary mx-2 rounded-md px-4 py-2 ${active ? '' : 'opacity-50'}`}
>
    <div class="flex">
        <h4 class="h4 grow"><strong>{folderName}</strong></h4>
        <button
            class="btn-icon rounded-none hover:variant-filled-surface"
            onclick={() => deleteAudioSource(source.path)}
        >
            <Trash2 />
        </button>
    </div>
    <div class="grid grid-cols-[1fr_auto] gap-y-2">
        <p>Active</p>
        <Switch
            name="active"
            bind:checked={active}
            onCheckedChange={(state) =>
                updateAudioSource(
                    source.path,
                    source.path,
                    state.checked,
                    recursive
                )}
        />
        <p>Include subfolders</p>
        <Switch
            name="recursive"
            bind:checked={recursive}
            onCheckedChange={(state) =>
                updateAudioSource(
                    source.path,
                    source.path,
                    active,
                    state.checked
                )}
        />
    </div>
</div>
