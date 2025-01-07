<script lang="ts">
    import {
        Pause,
        Play,
        Repeat,
        Shuffle,
        SkipBack,
        SkipForward
    } from 'lucide-svelte'
    import {
        globalGuild,
        trackQueue,
        playerState,
        recentlyPlayed
    } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        LOOP_TRACK,
        PAUSE_PLAYBACK,
        QUEUE_TRACK,
        RESUME_PLAYBACK,
        SEEK_TRACK,
        SKIP_TRACK
    } from './events'
    import { Progress } from '@skeletonlabs/skeleton-svelte'
    import { onDestroy } from 'svelte'
    import type { Track } from './types'

    function formatSeconds(seconds: number): string {
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

    async function handleBackSkip() {
        // If the queue is empty, do nothing
        if (trackQueue.tracks.length === 0) {
            return
        }

        if (
            playerState.position >= 5 /* seconds */ ||
            !recentlyPlayed.tracks[0]
        ) {
            await emit(SEEK_TRACK, { guildId: globalGuild.id, position: 0 })
        } else {
            // Get previous track, if any
            // Remove previous track from recents
            // Prepend previous track to queue
            const mostRecent = recentlyPlayed.tracks.shift() as Track
            trackQueue.tracks.unshift(mostRecent)
            await emit(QUEUE_TRACK, {
                guildId: globalGuild.id,
                trackData: mostRecent,
                looping: playerState.looping,
                prepend: true,
                overwrite: false
            })
        }
    }

    async function handlePlayerBarClick(
        e: MouseEvent & {
            currentTarget: EventTarget & HTMLButtonElement
        }
    ) {
        const rect = e.currentTarget.getBoundingClientRect()
        const clickX = e.clientX - rect.left
        const progressWidth = rect.width
        const seekTo = Math.floor((clickX / progressWidth) * duration)
        await emit(SEEK_TRACK, { guildId: globalGuild.id, position: seekTo })
    }

    function rgbToHex(rgb: string): string {
        const [r, g, b] = rgb.split(' ').map(Number)
        const toHex = (value: number) => value.toString(16).padStart(2, '0')
        return `#${toHex(r)}${toHex(g)}${toHex(b)}`
    }

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )

    let duration = $derived(trackQueue.tracks[0]?.duration ?? 60)

    let fmtProgress = $derived(formatSeconds(playerState.position))
    let fmtDuration = $derived(formatSeconds(duration))

    // Interval IDs are kept in an array just in case something bugs out
    // so that it won't overwrite the previous ID and leave an eternal leaked
    // interval
    let timerIds: number[] = []
    $effect(() => {
        if (playerState.playing) {
            const timerId = setInterval(() => {
                playerState.position += 1
            }, 1000)
            timerIds.push(timerId)
        } else {
            timerIds.forEach(clearInterval)
        }
    })

    onDestroy(() => timerIds.forEach(clearInterval))
</script>

<div class="border-t-[1px] border-surface-900 mt-2 h-24 p-2 flex-none">
    <div class="flex justify-center gap-2 mb-3 mt-2">
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            disabled><Shuffle /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={handleBackSkip}><SkipBack /></button
        >
        {#if playerState.playing}
            <button
                class="btn-icon rounded-none preset-filled-primary-100-900"
                onclick={async () => {
                    await emit(PAUSE_PLAYBACK, { guildId: globalGuild.id })
                }}
            >
                <Pause /></button
            >
        {:else}
            <button
                class="btn-icon rounded-none preset-filled-primary-100-900"
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
        <button
            class="w-full"
            onclick={handlePlayerBarClick}
            aria-label="player-bar"
        >
            <Progress
                max={duration}
                meterBg="bg-white hover:bg-primary-400-600"
                trackBg="bg-surface-200-800 hover:bg-surface-300-700"
                height="h-1"
                value={playerState.position}
            />
        </button>
        <p class="type-scale-2 opacity-70">
            {#if trackQueue.tracks[0]}{fmtDuration}{:else}0:00{/if}
        </p>
    </div>
</div>

<!-- <progress
    value={playerState.position}
    max={duration}
    class="w-full h-1 progress"
></progress> -->
