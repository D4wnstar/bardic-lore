<script>
	import { Search } from 'lucide-svelte'
</script>

<div id="tag-sidebar">
	<div class="bg-surface-200-700-token flex items-center space-x-2 px-2">
		<Search />
		<input
			type="search"
			class="h-12 w-full rounded-none border-none bg-transparent focus:ring-0"
			placeholder="Search tags..."
		/>
	</div>

	<div class="my-2 flex items-center justify-center gap-2">
		<button class="variant-soft-primary btn">ALL</button>
		|
		<button class="variant-soft-primary btn opacity-40">ANY</button>
	</div>

	<div class="mx-2 space-y-2">
		<div class="bg-secondary-50-900-token rounded-md p-2">
			<strong>Selected</strong>
			<div>tags here</div>
		</div>
		<div class="bg-tertiary-50-900-token rounded-md p-2">
			<strong>Available</strong>
			<div>tags here</div>
		</div>
	</div>
</div>
