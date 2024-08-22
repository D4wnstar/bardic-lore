<script>
	import { ListMusic, PlaySquare } from 'lucide-svelte'
</script>

<div
	class="variant-soft-surface flex min-w-[15rem] flex-col items-center rounded-md p-2 text-center"
>
	<div class="grow">
		<h3 class="h3">Currently Playing</h3>
	</div>
	<div class="flex gap-2">
		<button class="variant-filled-primary btn-icon rounded-none hover:variant-filled-primary"
			><PlaySquare size="32" /></button
		>
		<button class="btn-icon rounded-none hover:variant-filled-primary"
			><ListMusic size="32" /></button
		>
	</div>
</div>
