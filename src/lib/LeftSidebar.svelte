<script lang="ts">
    import { Bot, Folder, PanelRightOpen, Settings, Tag } from 'lucide-svelte'
    import Tags from '$lib/left-sidebar-tabs/Tags.svelte'
    import AudioSources from '$lib/left-sidebar-tabs/AudioSources.svelte'
    import BotControls from './left-sidebar-tabs/BotControls.svelte'
    import AppSettings from './left-sidebar-tabs/AppSettings.svelte'
    import { fade } from 'svelte/transition'
    import { expoIn } from 'svelte/easing'
    import { onMount } from 'svelte'
    import type { CachedTrack } from './types'
    import { settings } from './stores.svelte'
    import { TagSet } from './state.svelte'
    import ButtonWithTooltip from './popovers/ButtonWithTooltip.svelte'

    interface Props {
        addTrack: (track: CachedTrack) => void
        removeTrack: (track: CachedTrack) => void
        getCachedTracks: () => Promise<void>
        filterTracks: () => void
        selectedTags: TagSet
        tagsMode: 'any' | 'all'
    }

    let {
        addTrack,
        removeTrack,
        getCachedTracks,
        filterTracks,
        selectedTags = $bindable(new TagSet([])),
        tagsMode = $bindable('all')
    }: Props = $props()

    let sidebarVisible: boolean = $state(true)
    let tabIndex: number = $state(1)
    let tooltipOpen: boolean[] = $state([false, false, false, false, false])

    let minWidth = $state(300)

    let id: number[] = []
    function openCloseSidebar() {
        sidebarVisible = !sidebarVisible
        const newId = setInterval(() => {
            if (!sidebarVisible) {
                minWidth -= 35
            } else {
                minWidth += 35
            }
        }, 10)
        id.push(newId)
    }

    $effect(() => {
        if (minWidth < 52) {
            minWidth = 52
            id.forEach(clearInterval)
        } else if (minWidth > 300) {
            minWidth = 300
            id.forEach(clearInterval)
        }
    })

    let previousWidth = window.innerWidth
    onMount(() => {
        window.addEventListener('resize', () => {
            if (
                previousWidth > 1024 &&
                window.innerWidth < 1024 &&
                sidebarVisible &&
                settings.autohideSidebars
            ) {
                openCloseSidebar()
            }
            if (
                previousWidth < 1024 &&
                window.innerWidth > 1024 &&
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
    class="my-2 max-w-[300px] border-r-[1px] border-surface-100-900 px-2 flex flex-col overflow-hidden"
    style={`min-width: ${minWidth}px`}
>
    <div class="flex w-full gap-1 pb-2">
        <ButtonWithTooltip
            bind:open={tooltipOpen[0]}
            tooltip={sidebarVisible ? 'Hide sidebar' : 'Show sidebar'}
            classes="hover:preset-filled-surface-500"
            onclick={openCloseSidebar}><PanelRightOpen /></ButtonWithTooltip
        >
        {#if sidebarVisible}
            <ButtonWithTooltip
                bind:open={tooltipOpen[1]}
                tooltip="Tags"
                classes={tabIndex === 1
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 1)}><Tag /></ButtonWithTooltip
            >
            <ButtonWithTooltip
                bind:open={tooltipOpen[2]}
                tooltip="Audio Sources"
                classes={tabIndex === 2
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 2)}><Folder /></ButtonWithTooltip
            >
            <ButtonWithTooltip
                bind:open={tooltipOpen[3]}
                tooltip="Bot Controls"
                classes={tabIndex === 3
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 3)}><Bot /></ButtonWithTooltip
            >
            <ButtonWithTooltip
                bind:open={tooltipOpen[4]}
                tooltip="App Settings"
                classes={tabIndex === 4
                    ? 'preset-filled-primary-500'
                    : 'hover:preset-filled-primary-500'}
                onclick={() => (tabIndex = 4)}><Settings /></ButtonWithTooltip
            >
        {/if}
    </div>
    <hr class="hr pb-2" />

    {#if sidebarVisible}
        <div
            class="overflow-x-auto"
            transition:fade={{
                duration: sidebarVisible ? 200 : 10,
                easing: expoIn
            }}
        >
            {#if tabIndex === 1}
                <Tags bind:selectedTags bind:tagsMode {filterTracks} />
            {:else if tabIndex === 2}
                <AudioSources {addTrack} {removeTrack} {getCachedTracks} />
            {:else if tabIndex === 3}
                <BotControls />
            {:else if tabIndex === 4}
                <AppSettings {getCachedTracks} />
            {/if}
        </div>
    {/if}
</aside>
