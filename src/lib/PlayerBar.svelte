<script>
	import { Play, Repeat, Shuffle, SkipBack, SkipForward } from 'lucide-svelte'
</script>

<div class="variant-soft-surface flex p-2 mt-2 gap-2 rounded-md justify-center">
	<button class="btn-icon rounded-none hover:variant-filled-surface"><Shuffle /></button>
	<button class="btn-icon rounded-none hover:variant-filled-surface"><SkipBack /></button>
	<button class="btn-icon rounded-none hover:variant-filled-surface"
		><Play color="#2161b8" /></button
	>
	<button class="btn-icon rounded-none hover:variant-filled-surface"><SkipForward /></button>
	<button class="btn-icon rounded-none hover:variant-filled-surface"><Repeat /></button>
</div>
