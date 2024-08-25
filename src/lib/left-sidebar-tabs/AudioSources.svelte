<script lang="ts">
	import { invoke } from '@tauri-apps/api/core'
	import { type AudioSource } from '$lib/types'
	import SourceBox from './SourceBox.svelte'
	import { getToastStore } from '@skeletonlabs/skeleton'
	import { onMount } from 'svelte'

	const toastStore = getToastStore()

	async function selectFolder() {
		let audioSources = await invoke<AudioSource[]>('add_audio_sources').catch((reason) => {
			console.error(reason)
			toastStore.trigger({
				message: reason,
				background: 'variant-filled-error'
			})
		})
		if (audioSources) {
			audioSources.forEach((s) => {
				if (!sources.find((s1) => s1.path == s.path)) {
					sources.push(s)
				}
			})
		}
		sources = sources
	}

	function handleSourcesUpdate(event: any) {
		sources = event.detail.sources
	}

	let sources: AudioSource[] = []

	onMount(async () => {
		sources = await invoke<AudioSource[]>('get_audio_sources').catch((err) => {
			console.error(`Error while getting audio sources. Error: ${err}`)
			return []
		})
	})
</script>

<div id="sources-sidebar" class="flex flex-col space-y-2">
	<h3 class="h3 text-center">Audio Sources</h3>
	<small class="small px-2 text-center">
		Audio Sources are the folders in which your audio files are contained. Audio files in these
		folders (and optionally sub-folders) will appear in the menu.
	</small>
	{#each sources as source}
		<SourceBox {source} on:updatedSources={(e) => handleSourcesUpdate(e)} />
	{/each}

	<div class="mx-2 flex justify-center gap-x-2">
		<button class="variant-ghost-primary btn self-center" on:click={selectFolder}>
			Add sources
		</button>
		<button class="variant-ghost-primary btn self-center">Refresh files</button>
	</div>
</div>
