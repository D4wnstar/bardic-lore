<script>
    import {
        Pause,
        Play,
        Repeat,
        Shuffle,
        SkipBack,
        SkipForward
    } from 'lucide-svelte'
    import { globalGuildId, currentTrack } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        LOOP_TRACK,
        PAUSE_PLAYBACK,
        RESUME_PLAYBACK,
        SKIP_TRACK
    } from './events'

    const activeColor = '#2161b8'
</script>

<div class="preset-outlined-surface-500 mt-2 h-24 rounded-md p-2 flex-none">
    <div class="flex justify-center gap-2">
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            disabled><Shuffle /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            disabled><SkipBack /></button
        >
        {#if currentTrack.playing}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-500"
                onclick={async () => {
                    await emit(PAUSE_PLAYBACK, { guildId: globalGuildId.id })
                }}
            >
                <Pause color={activeColor} size="32" /></button
            >
        {:else}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-500"
                onclick={async () => {
                    await emit(RESUME_PLAYBACK, { guildId: globalGuildId.id })
                }}
            >
                <Play color={activeColor} size="32" /></button
            >
        {/if}
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            onclick={async () => {
                await emit(SKIP_TRACK, { guildId: globalGuildId.id })
            }}><SkipForward /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            onclick={async () => {
                await emit(LOOP_TRACK, { guildId: globalGuildId.id })
            }}
            ><Repeat
                color={currentTrack.looping ? activeColor : '#ffffff'}
            /></button
        >
    </div>
</div>
