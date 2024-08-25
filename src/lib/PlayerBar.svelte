<script>
	import { Play, Repeat, Shuffle, SkipBack, SkipForward } from 'lucide-svelte'
	import { invoke } from '@tauri-apps/api/core'
</script>

<div class="variant-soft-surface mt-2 max-h-24 min-h-24 rounded-md p-2">
	<div class="flex justify-center gap-2">
		<button class="btn-icon rounded-none hover:variant-filled-surface"><Shuffle /></button>
		<button class="btn-icon rounded-none hover:variant-filled-surface"><SkipBack /></button>
		<button
			class="btn-icon rounded-none hover:variant-filled-surface"
			on:click={() => invoke('play')}
		>
			<Play color="#2161b8" size="32" /></button
		>
		<button class="btn-icon rounded-none hover:variant-filled-surface"><SkipForward /></button>
		<button class="btn-icon rounded-none hover:variant-filled-surface"><Repeat /></button>
	</div>
</div>
