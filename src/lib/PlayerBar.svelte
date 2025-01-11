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
        LOOP_TRACK,
        PAUSE_PLAYBACK,
        QUEUE_TRACK,
        RESUME_PLAYBACK,
        SEEK_TRACK,
        SKIP_TRACK
    } from './events'
    import type { Track } from './types'
    import VolumeSlider from './utils/VolumeSlider.svelte'
    import TrackProgressBar from './utils/TrackProgressBar.svelte'
    import { rgbToHex } from './utils/utils'

    async function handleBackSkip() {
        if (appState.playlist.isEmpty()) {
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

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )
</script>

<div
    class="border-t-[1px] border-surface-900 mt-2 h-24 p-2 flex-none flex items-center"
>
    <div class="2xl:min-w-[200px]"></div>
    <div class="grow">
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
                    color={appState.mainPlayer.looping
                        ? activeColor
                        : '#ffffff'}
                /></button
            >
        </div>

        <!-- Empty padding space. Width should be equal to volume slider -->
        <TrackProgressBar
            player={appState.mainPlayer}
            duration={appState.playlist.current()?.duration}
        />
    </div>

    <VolumeSlider player={appState.mainPlayer} classes="pt-4" />
</div>
