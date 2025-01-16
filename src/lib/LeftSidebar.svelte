<script lang="ts">
    import { Bot, Folder, PanelRightOpen, Settings, Tag } from 'lucide-svelte'
    import Tags from '$lib/left-sidebar-tabs/Tags.svelte'
    import AudioSources from '$lib/left-sidebar-tabs/AudioSources.svelte'
    import BotControls from './left-sidebar-tabs/BotControls.svelte'
    import AppSettings from './left-sidebar-tabs/AppSettings.svelte'
    import { fade, slide } from 'svelte/transition'
    import { expoIn } from 'svelte/easing'
    import { onMount } from 'svelte'

    interface Props {
        getTracks: Function
    }
    let { getTracks }: Props = $props()

    let visible: boolean = $state(true)
    let tabIndex: number = $state(3)

    let minWidth = $state(300)

    let id: number[] = []
    function openCloseSidebar() {
        visible = !visible
        const newId = setInterval(() => {
            if (!visible) {
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
            if (previousWidth > 1024 && window.innerWidth < 1024 && visible) {
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
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            onclick={openCloseSidebar}><PanelRightOpen /></button
        >
        {#if visible}
            <button
                class={`btn-icon rounded-none ${tabIndex === 1 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 1)}
                transition:slide={{ axis: 'x', duration: 100 }}><Tag /></button
            >
            <button
                class={`btn-icon rounded-none ${tabIndex === 2 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 2)}
                transition:slide={{ axis: 'x', duration: 100 }}
                ><Folder /></button
            >
            <button
                class={`btn-icon rounded-none ${tabIndex === 3 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 3)}
                transition:slide={{ axis: 'x', duration: 100 }}><Bot /></button
            >
            <button
                class={`btn-icon rounded-none ${tabIndex === 4 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
                onclick={() => (tabIndex = 4)}
                transition:slide={{ axis: 'x', duration: 100 }}
                ><Settings /></button
            >
        {/if}
    </div>
    <hr class="hr" />

    {#if visible}
        <div
            class="overflow-x-auto"
            transition:fade={{ duration: visible ? 200 : 10, easing: expoIn }}
        >
            {#if tabIndex === 1}
                <Tags />
            {:else if tabIndex === 2}
                <AudioSources {getTracks} />
            {:else if tabIndex === 3}
                <BotControls />
            {:else if tabIndex === 4}
                <AppSettings />
            {/if}
        </div>
    {/if}
</aside>
