<script lang="ts">
	import type { AudioSource } from '$lib/types'
	import { basename } from '@tauri-apps/api/path'
	import { createEventDispatcher, onMount } from 'svelte'
	import { getToastStore, SlideToggle } from '@skeletonlabs/skeleton'
	import { invoke } from '@tauri-apps/api/core'
	import { Trash2 } from 'lucide-svelte'

	export let source: AudioSource

	const toastStore = getToastStore()
	const dispatch = createEventDispatcher()

	async function updateAudioSource(
		oldPath: string,
		path: string,
		active: boolean,
		recursive: boolean
	) {
		let audioSources = await invoke<AudioSource[]>('update_audio_source', {
			oldPath: oldPath,
			path: path,
			active: active,
			recursive: recursive
		}).catch((reason) => {
			console.error(reason)
			toastStore.trigger({
				message: reason,
				background: 'variant-filled-error'
			})
		})
		// console.log(audioSources)
		dispatch('updatedSources', {
			sources: audioSources
		})
	}

	async function deleteAudioSource(path: string) {
		let audioSources = await invoke<AudioSource[]>('delete_audio_source', { path: path }).catch(
			(reason) => {
				console.error(reason)
				toastStore.trigger({
					message: reason,
					background: 'variant-filled-error'
				})
			}
		)
		// console.log(audioSources)
		dispatch('updatedSources', {
			sources: audioSources
		})
	}

	let folderName = ''
	onMount(async () => (folderName = await basename(source.path)))

	let active = source.active
	let recursive = source.recursive
</script>

<div class={`variant-soft-secondary mx-2 rounded-md px-4 py-2 ${active ? '' : 'opacity-50'}`}>
	<div class="flex">
		<h4 class="h4 grow"><strong>{folderName}</strong></h4>
		<button
			class="btn-icon rounded-none hover:variant-filled-surface"
			on:click={() => deleteAudioSource(source.path)}
		>
			<Trash2 />
		</button>
	</div>
	<div class="grid grid-cols-[1fr_auto] gap-y-2">
		<p>Active</p>
		<SlideToggle
			name="active"
			bind:checked={active}
			active="bg-secondary-500"
			size="sm"
			on:change={() => updateAudioSource(source.path, source.path, active, recursive)}
		/>
		<p>Include subfolders</p>
		<SlideToggle
			name="recursive"
			bind:checked={recursive}
			active="bg-secondary-500"
			size="sm"
			on:change={() => updateAudioSource(source.path, source.path, active, recursive)}
		/>
	</div>
</div>
