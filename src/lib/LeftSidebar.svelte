<script lang="ts">
    import { Bot, Folder, PanelRightOpen, Settings, Tag } from 'lucide-svelte'
    import Tags from '$lib/left-sidebar-tabs/Tags.svelte'
    import AudioSources from '$lib/left-sidebar-tabs/AudioSources.svelte'
    import BotSettings from './left-sidebar-tabs/BotSettings.svelte'
    import AppSettings from './left-sidebar-tabs/AppSettings.svelte'

    interface Props {
        getTracks: Function
    }
    let { getTracks }: Props = $props()

    let tabIndex: number = $state(3)
</script>

<aside
    class="preset-filled-surface h-full my-2 min-w-[300px] max-w-[300px] border-r-[1px] border-surface-100-900 px-2 flex flex-col overflow-hidden"
>
    <div class="flex w-full gap-1 min-h-12">
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><PanelRightOpen /></button
        >
        <button
            class={`btn-icon rounded-none ${tabIndex === 1 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 1)}><Tag /></button
        >
        <button
            class={`btn-icon rounded-none ${tabIndex === 2 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 2)}><Folder /></button
        >
        <button
            class={`btn-icon rounded-none ${tabIndex === 3 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 3)}><Bot /></button
        >
        <button
            class={`btn-icon rounded-none ${tabIndex === 4 ? 'preset-filled-primary-500' : 'hover:preset-filled-primary-500'}`}
            onclick={() => (tabIndex = 4)}><Settings /></button
        >
    </div>

    {#if tabIndex === 1}
        <Tags />
    {:else if tabIndex === 2}
        <AudioSources {getTracks} />
    {:else if tabIndex === 3}
        <BotSettings />
    {:else if tabIndex === 4}
        <AppSettings />
    {/if}
</aside>
