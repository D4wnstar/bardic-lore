<script>
	import { Bot, PanelRightOpen, Search, Settings, Tag } from 'lucide-svelte'
</script>

<aside class="variant-soft-surface h-screen">
	<div class="p-2 w-full flex gap-1">
		<button class="btn-icon rounded-none hover:variant-filled-surface"><PanelRightOpen /></button>
		<button class="btn-icon rounded-none hover:variant-filled-primary variant-filled-primary"
			><Tag /></button
		>
		<button class="btn-icon rounded-none hover:variant-filled-primary"><Bot /></button>
		<button class="btn-icon rounded-none hover:variant-filled-primary"><Settings /></button>
	</div>

	<div class="flex space-x-2 items-center px-2 bg-surface-200-700-token">
		<Search />
		<input
			type="search"
			class="bg-transparent border-none w-full h-12 rounded-none focus:ring-0"
			placeholder="Search tags..."
		/>
	</div>

	<div class="my-2 flex items-center justify-center gap-2">
		<button class="btn variant-soft-primary">ALL</button>
		|
		<button class="btn variant-soft-primary opacity-40">ANY</button>
	</div>

	<div class="space-y-2 mx-1">
		<div class="bg-secondary-50-900-token p-2 rounded-md">
			<strong>Selected</strong>
			<div>tags here</div>
		</div>
		<div class="bg-tertiary-50-900-token p-2 rounded-md">
			<strong>Available</strong>
			<div>tags here</div>
		</div>
	</div>
</aside>
