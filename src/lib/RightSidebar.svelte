<script>
    import { ListMusic, PanelLeftOpen, PlaySquare } from 'lucide-svelte'
    import Queue from './right-sidebar-tabs/Queue.svelte'
    let tabIndex = $state(1)
</script>

<aside
    class="preset-filled-surface h-full min-w-[300px] max-w-[300px] my-2 border-l-[1px] border-surface-100-900 px-2 flex flex-col overflow-hidden"
>
    <div class="flex gap-1 min-h-12 self-end">
        <button
            class={`btn-icon rounded-none ${tabIndex === 1 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 1)}><ListMusic /></button
        >
        <button
            class={`btn-icon rounded-none ${tabIndex === 2 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 2)}><PlaySquare /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><PanelLeftOpen /></button
        >
    </div>

    {#if tabIndex === 1}
        <Queue />
    {:else if tabIndex === 2}
        <h3 class="h3">Currently playing</h3>
    {/if}
</aside>
