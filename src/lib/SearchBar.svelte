<script lang="ts">
    import { Search } from 'lucide-svelte'

    interface Props {
        filterTracks: (searchTerm: string) => void
    }

    let { filterTracks }: Props = $props()
</script>

<header
    class="bg-surface-100-900 mx-auto mb-4 flex h-12 w-1/2 min-w-[300px] max-w-[600px] items-center justify-center gap-2 rounded-md px-2"
>
    <Search />
    <input
        type="search"
        oninput={(e) => filterTracks(e.currentTarget.value)}
        class="h-12 grow border-none bg-transparent focus:ring-0"
        placeholder="Search songs..."
    />
</header>
