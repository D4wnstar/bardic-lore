<script lang="ts">
    import { Search } from 'lucide-svelte'
    import { appState, appTracks, selectedTags } from './stores.svelte'
</script>

<header
    class="bg-surface-100-900 mx-auto mb-4 flex h-12 w-1/2 min-w-[300px] max-w-[600px] items-center justify-center gap-2 rounded-md px-2"
>
    <Search />
    <input
        type="search"
        oninput={(_) =>
            appTracks.filter(
                selectedTags,
                appState.tagsMode,
                appState.trackSearchTerm
            )}
        bind:value={appState.trackSearchTerm}
        class="h-12 grow border-none bg-transparent focus:ring-0"
        placeholder="Search songs..."
    />
</header>
