<script>
    import {
        Pause,
        Play,
        Repeat,
        Shuffle,
        SkipBack,
        SkipForward
    } from 'lucide-svelte'
    import { globalGuildId, currentSong } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
</script>

<div class="preset-outlined-surface-500 mt-2 h-24 rounded-md p-2 flex-none">
    <div class="flex justify-center gap-2">
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><Shuffle /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><SkipBack /></button
        >
        {#if currentSong.playing}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-500"
                onclick={async () => {
                    await emit('pause-playback', { guildId: globalGuildId.id })
                }}
            >
                <Pause color="#2161b8" size="32" /></button
            >
        {:else}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-500"
                onclick={async () => {
                    await emit('start-playback', { guildId: globalGuildId.id })
                }}
            >
                <Play color="#2161b8" size="32" /></button
            >
        {/if}
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><SkipForward /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><Repeat /></button
        >
    </div>
</div>
