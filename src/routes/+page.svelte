<script>
	import { Search } from 'lucide-svelte'
	import SongBox from '$lib/SongBox.svelte'
	import CurrentlyPlaying from '$lib/CurrentlyPlaying.svelte'
	import PlayerBar from '$lib/PlayerBar.svelte'
</script>

<main class="p-4 flex flex-col">
	<div class="grow flex">
		<div class="grow flex flex-col">
			<header
				class="flex items-center justify-center mx-auto w-1/2 min-w-[300px] max-w-[600px] px-2 gap-2 bg-surface-200-700-token rounded-md"
			>
				<Search />
				<input
					type="search"
					class="bg-transparent border-none grow h-12 rounded-none focus:ring-0"
					placeholder="Search songs..."
				/>
			</header>
			<section class="flex flex-wrap grow gap-2 mr-4 mt-4 overflow-auto">
				<SongBox tags="[Pop], [Rock], [Hip Hop], [Hip Hop]" />
				<SongBox tags="[R&B], [Jazz], [Blues]" />
				<SongBox tags="[Country]" />
				<SongBox
					tags="[Electronic], [Dance], [Techno], [Electronic], [Dance], [Techno], [Electronic], [Dance], [Techno]"
				/>
				<SongBox tags="[Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
				<SongBox tags="[Reggae], [Ska], [Dub], [Classical], [Orchestral]" />
			</section>
		</div>
		<CurrentlyPlaying />
	</div>

	<PlayerBar />
</main>
