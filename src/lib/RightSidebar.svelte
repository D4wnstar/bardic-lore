<script lang="ts">
    import { ListMusic, PanelLeftOpen, PlaySquare } from 'lucide-svelte'
    import Queue from './right-sidebar-tabs/Queue.svelte'
    import { fade, slide } from 'svelte/transition'
    import { expoIn } from 'svelte/easing'
    import CurrentlyPlaying from './right-sidebar-tabs/CurrentlyPlaying.svelte'

    let visible = $state(true)
    let tabIndex = $state(2)

    let minWidth = $state(300)

    let id: number
    function closeSidebar() {
        visible = !visible
        id = setInterval(() => {
            if (!visible) {
                minWidth -= 35
            } else {
                minWidth += 35
            }
        }, 10)
    }

    $effect(() => {
        if (minWidth < 56) {
            minWidth = 56
            clearInterval(id)
        } else if (minWidth > 300) {
            minWidth = 300
            clearInterval(id)
        }
    })
</script>

<aside
    class="preset-filled-surface h-full my-2 max-w-[300px] border-l-[1px] border-surface-100-900 px-2 flex flex-col overflow-hidden"
    style={`min-width: ${minWidth}px`}
>
    <div class="flex gap-1 min-h-12 self-end">
        {#if visible}
            <button
                class={`btn-icon rounded-none ${tabIndex === 1 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 1)}
                transition:slide={{ axis: 'x', duration: 100 }}
                ><ListMusic /></button
            >
            <button
                class={`btn-icon rounded-none ${tabIndex === 2 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 2)}
                transition:slide={{ axis: 'x', duration: 100 }}
                ><PlaySquare /></button
            >
        {/if}
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            onclick={closeSidebar}><PanelLeftOpen /></button
        >
    </div>

    {#if visible}
        <div
            class="overflow-auto"
            transition:fade={{ duration: visible ? 200 : 10, easing: expoIn }}
        >
            {#if tabIndex === 1}
                <Queue />
            {:else if tabIndex === 2}
                <CurrentlyPlaying />
            {/if}
        </div>
    {/if}
</aside>
