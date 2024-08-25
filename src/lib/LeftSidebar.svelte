<script lang="ts">
	import { Bot, Folder, PanelRightOpen, Settings, Tag } from 'lucide-svelte'
	import Tags from './left-sidebar-tabs/Tags.svelte'
	import AudioSources from './left-sidebar-tabs/AudioSources.svelte'

	let tabIndex: number = 1
</script>

<aside class="variant-soft-surface h-screen w-[300px]">
	<div class="flex w-full gap-1 p-2">
		<button class="btn-icon rounded-none hover:variant-filled-surface"><PanelRightOpen /></button>
		<button
			class={`btn-icon rounded-none ${tabIndex === 1 ? 'variant-filled-primary' : 'hover:variant-filled-primary'}`}
			on:click={() => (tabIndex = 1)}><Tag /></button
		>
		<button
			class={`btn-icon rounded-none ${tabIndex === 2 ? 'variant-filled-primary' : 'hover:variant-filled-primary'}`}
			on:click={() => (tabIndex = 2)}><Folder /></button
		>
		<button
			class={`btn-icon rounded-none ${tabIndex === 3 ? 'variant-filled-primary' : 'hover:variant-filled-primary'}`}
			on:click={() => (tabIndex = 3)}><Bot /></button
		>
		<button
			class={`btn-icon rounded-none ${tabIndex === 4 ? 'variant-filled-primary' : 'hover:variant-filled-primary'}`}
			on:click={() => (tabIndex = 4)}><Settings /></button
		>
	</div>

	{#if tabIndex === 1}
		<Tags />
	{:else if tabIndex === 2}
		<AudioSources />
	{:else if tabIndex === 3}
		<!-- Add content for Bot settings here -->
		<span>unimplemented bot</span>
	{:else if tabIndex === 4}
		<!-- Add content for App settings here -->
		<span>unimplemented settings</span>
	{/if}
</aside>
