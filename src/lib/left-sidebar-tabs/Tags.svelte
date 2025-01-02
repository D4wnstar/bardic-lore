<script>
    import { Search } from 'lucide-svelte'
</script>

<div id="tag-sidebar" class="flex flex-col h-full min-h-0">
    <h3 class="h3 text-center">Tags</h3>
    <div class="flex-none bg-surface-200-800 flex items-center space-x-2 px-2">
        <Search />
        <input
            type="search"
            class="h-12 w-full rounded-none border-none bg-transparent focus:ring-0"
            placeholder="Search tags..."
        />
    </div>

    <div class="my-2 flex items-center justify-center gap-2 flex-none">
        <button class="preset-outlined-primary-500 btn">ALL</button>
        |
        <button class="preset-outlined-primary-500 btn opacity-40">ANY</button>
    </div>

    <div class="flex-1 overflow-auto min-h-0 mx-2 space-y-2">
        <div class="bg-secondary-50-950 rounded-md p-2">
            <strong>Selected</strong>
            <div>tags here</div>
        </div>
        <div class="bg-tertiary-50-950 rounded-md p-2">
            <strong>Available</strong>
            <div>tags here</div>
        </div>
    </div>
</div>
