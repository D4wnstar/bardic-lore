<script lang="ts">
    import { ListMusic, PanelLeftOpen, PlaySquare } from 'lucide-svelte'
    import Queue from './right-sidebar-tabs/Queue.svelte'
    import { fade } from 'svelte/transition'
    import { expoIn } from 'svelte/easing'
    import CurrentlyPlaying from './right-sidebar-tabs/CurrentlyPlaying.svelte'
    import { onMount } from 'svelte'
    import { settings } from './stores.svelte'
    import ButtonWithTooltip from './popovers/ButtonWithTooltip.svelte'

    let sidebarVisible = $state(true)
    let tabIndex = $state(1)

    let minWidth = $state(300)
    let tooltipOpen: boolean[] = $state([false, false, false])

    let id: number
    function openCloseSidebar() {
        sidebarVisible = !sidebarVisible
        id = setInterval(() => {
            if (!sidebarVisible) {
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

    let previousWidth = window.innerWidth
    onMount(() => {
        window.addEventListener('resize', () => {
            if (
                previousWidth > 900 &&
                window.innerWidth < 900 &&
                sidebarVisible &&
                settings.autohideSidebars
            ) {
                openCloseSidebar()
            }
            if (
                previousWidth < 900 &&
                window.innerWidth > 900 &&
                !sidebarVisible &&
                settings.autohideSidebars
            ) {
                openCloseSidebar()
            }
            previousWidth = window.innerWidth
        })
    })
</script>

<aside
    class="preset-filled-surface h-full my-2 max-w-[300px] border-l-[1px] border-surface-100-900 px-2 flex flex-col overflow-hidden"
    style={`min-width: ${minWidth}px`}
>
    <div class="flex gap-1 min-h-12 self-end">
        {#if sidebarVisible}
            <ButtonWithTooltip
                bind:open={tooltipOpen[0]}
                tooltip="Queue"
                classes={tabIndex === 1
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 1)}><ListMusic /></ButtonWithTooltip
            >
            <ButtonWithTooltip
                bind:open={tooltipOpen[1]}
                tooltip="Currently Playing"
                classes={tabIndex === 2
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 2)}><PlaySquare /></ButtonWithTooltip
            >
        {/if}
        <ButtonWithTooltip
            bind:open={tooltipOpen[2]}
            tooltip={sidebarVisible ? 'Hide sidebar' : 'Show sidebar'}
            classes="hover:preset-filled-surface-500"
            onclick={openCloseSidebar}><PanelLeftOpen /></ButtonWithTooltip
        >
    </div>

    {#if sidebarVisible}
        <div
            class="overflow-auto"
            transition:fade={{
                duration: sidebarVisible ? 200 : 10,
                easing: expoIn
            }}
        >
            {#if tabIndex === 1}
                <Queue />
            {:else if tabIndex === 2}
                <CurrentlyPlaying />
            {/if}
        </div>
    {/if}
</aside>
