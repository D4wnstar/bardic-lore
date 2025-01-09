<script lang="ts">
    import {
        Pause,
        Play,
        Repeat,
        Shuffle,
        SkipBack,
        SkipForward
    } from 'lucide-svelte'
    import { appState } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        CHANGE_VOLUME,
        LOOP_TRACK,
        MUTE_UNMUTE,
        PAUSE_PLAYBACK,
        QUEUE_TRACK,
        RESUME_PLAYBACK,
        SEEK_TRACK,
        SKIP_TRACK
    } from './events'
    import { onDestroy } from 'svelte'
    import type { Track } from './types'
    import VolumeSlider from './utils/VolumeSlider.svelte'
    import TrackProgressBar from './utils/TrackProgressBar.svelte'
    import { rgbToHex } from './utils/utils'

    async function handleBackSkip() {
        // If the queue is empty, do nothing
        if (appState.trackQueue.length === 0) {
            return
        }

        if (
            appState.mainPlayer.position >= 5 /* seconds */ ||
            !appState.recentlyPlayed[0]
        ) {
            await emit(SEEK_TRACK, {
                guildId: appState.guildId,
                position: 0,
                parallel: false
            })
        } else {
            // Get previous track, if any
            // Remove previous track from recents
            // Prepend previous track to queue
            const mostRecent = appState.recentlyPlayed.shift() as Track
            // appState.trackQueue.unshift(mostRecent)
            await emit(QUEUE_TRACK, {
                guildId: appState.guildId,
                trackData: mostRecent,
                looping: appState.mainPlayer.looping,
                prepend: true,
                overwrite: false,
                volume: appState.mainPlayer.volume
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
        await emit(SEEK_TRACK, {
            guildId: appState.guildId,
            position: seekTo,
            parallel: false
        })
    }

    async function onVolumeChange() {
        await emit(CHANGE_VOLUME, {
            guildId: appState.guildId,
            volume: appState.mainPlayer.volume,
            parallel: false
        })
    }

    async function onMuteClick() {
        await emit(MUTE_UNMUTE, { guildId: appState.guildId, parallel: false })
    }

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )

    let duration = $derived(appState.trackQueue[0]?.duration ?? 60)

    // Interval IDs are kept in an array just in case something bugs out
    // so that it won't overwrite the previous ID and leave an eternal leaked
    // interval
    let timerIds: number[] = []
    $effect(() => {
        if (appState.mainPlayer.playing) {
            const timerId = setInterval(() => {
                appState.mainPlayer.position += 1
            }, 1000)
            timerIds.push(timerId)
        } else {
            timerIds.forEach(clearInterval)
        }
    })

    onDestroy(() => timerIds.forEach(clearInterval))
</script>

<div class="border-t-[1px] border-surface-900 mt-2 h-24 p-2 flex-none">
    <div class="flex justify-center gap-2 mb-2 mt-2">
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            disabled><Shuffle /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={handleBackSkip}><SkipBack /></button
        >
        {#if appState.mainPlayer.playing}
            <button
                class="btn-icon rounded-none preset-filled-primary-100-900"
                onclick={async () => {
                    await emit(PAUSE_PLAYBACK, {
                        guildId: appState.guildId,
                        parallel: false
                    })
                }}
            >
                <Pause /></button
            >
        {:else}
            <button
                class="btn-icon rounded-none preset-filled-primary-100-900"
                onclick={async () => {
                    await emit(RESUME_PLAYBACK, {
                        guildId: appState.guildId,
                        parallel: false
                    })
                }}
            >
                <Play /></button
            >
        {/if}
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={async () => {
                await emit(SKIP_TRACK, {
                    guildId: appState.guildId,
                    parallel: false
                })
            }}><SkipForward /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-100-900"
            onclick={async () => {
                appState.mainPlayer.looping = !appState.mainPlayer.looping
                await emit(LOOP_TRACK, {
                    guildId: appState.guildId,
                    parallel: false
                })
            }}
            ><Repeat
                color={appState.mainPlayer.looping ? activeColor : '#ffffff'}
            /></button
        >
    </div>

    <div class="flex">
        <!-- Empty padding space. Width should be equal to volume slider -->
        <div class="min-w-[200px]"></div>
        <TrackProgressBar
            player={appState.mainPlayer}
            duration={appState.trackQueue[0]?.duration}
            onBarClick={handlePlayerBarClick}
        />
        <VolumeSlider
            player={appState.mainPlayer}
            {onMuteClick}
            {onVolumeChange}
        />
    </div>
</div>
