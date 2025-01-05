<script lang="ts">
    import {
        Pause,
        Play,
        Repeat,
        Shuffle,
        SkipBack,
        SkipForward
    } from 'lucide-svelte'
    import { globalGuild, trackQueue, playerState } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        LOOP_TRACK,
        PAUSE_PLAYBACK,
        RESUME_PLAYBACK,
        SKIP_TRACK
    } from './events'
    import { Progress } from '@skeletonlabs/skeleton-svelte'
    import { onMount } from 'svelte'

    export function formatSeconds(seconds: number): string {
        const hours = Math.floor(seconds / 3600)
        const minutes = Math.floor((seconds % 3600) / 60)
        const remainingSeconds = seconds % 60

        const formattedHours = hours.toString().padStart(1, '0')
        const formattedMinutes = minutes.toString().padStart(1, '0')
        const formattedSeconds = remainingSeconds.toString().padStart(2, '0')

        if (hours > 0) {
            return `${formattedHours}:${formattedMinutes}:${formattedSeconds}`
        } else {
            return `${formattedMinutes}:${formattedSeconds}`
        }
    }

    let duration = $derived(trackQueue.tracks[0]?.duration ?? 100)

    let fmtProgress = $derived(formatSeconds(playerState.trackProgress))
    let fmtDuration = $derived(formatSeconds(duration))

    let timerId: number
    $effect(() => {
        if (playerState.playing) {
            timerId = setInterval(() => {
                playerState.trackProgress += 1
            }, 1000)
        } else {
            clearInterval(timerId)
        }
    })

    function rgbToHex(rgb: string): string {
        const [r, g, b] = rgb.split(' ').map(Number)
        const toHex = (value: number) => value.toString(16).padStart(2, '0')
        return `#${toHex(r)}${toHex(g)}${toHex(b)}`
    }

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )
</script>

<div class="border-t-[1px] border-surface-900 mt-2 h-24 p-2 flex-none">
    <div class="flex justify-center gap-2 mb-3 mt-2">
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            disabled><Shuffle /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            disabled><SkipBack /></button
        >
        {#if playerState.playing}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-100-900"
                onclick={async () => {
                    await emit(PAUSE_PLAYBACK, { guildId: globalGuild.id })
                }}
            >
                <Pause /></button
            >
        {:else}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-100-900"
                onclick={async () => {
                    await emit(RESUME_PLAYBACK, { guildId: globalGuild.id })
                }}
            >
                <Play /></button
            >
        {/if}
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={async () => {
                await emit(SKIP_TRACK, { guildId: globalGuild.id })
            }}><SkipForward /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={async () => {
                playerState.looping = !playerState.looping
                await emit(LOOP_TRACK, { guildId: globalGuild.id })
            }}
            ><Repeat
                color={playerState.looping ? activeColor : '#ffffff'}
            /></button
        >
    </div>
    <div class="flex items-center gap-4 px-4 max-w-[550px] mx-auto">
        <p class="type-scale-2 opacity-70">{fmtProgress}</p>
        <Progress
            max={duration}
            meterBg="bg-white hover:bg-primary-400-600"
            height="h-1"
            value={playerState.trackProgress}
        />
        <p class="type-scale-2 opacity-70">
            {#if playerState.playing}{fmtDuration}{:else}0:00{/if}
        </p>
    </div>
</div>
