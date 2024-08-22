<script>
	import { Search } from 'lucide-svelte'
	import SongBox from '$lib/SongBox.svelte'
	import CurrentlyPlaying from '$lib/CurrentlyPlaying.svelte'
	import PlayerBar from '$lib/PlayerBar.svelte'
</script>

<main class="flex max-h-screen flex-col p-4">
	<div class="flex grow">
		<div class="flex grow flex-col">
			<header
				class="bg-surface-200-700-token mx-auto mb-4 flex h-12 w-1/2 min-w-[300px] max-w-[600px] items-center justify-center gap-2 rounded-md px-2"
			>
				<Search />
				<input
					type="search"
					class="h-12 grow rounded-none border-none bg-transparent focus:ring-0"
					placeholder="Search songs..."
				/>
			</header>
			<section class="mr-4 flex max-h-[calc(100vh-192px)] flex-wrap gap-2 overflow-y-auto p-1">
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
			</section>
		</div>
		<CurrentlyPlaying />
	</div>

	<PlayerBar />
</main>
