<script lang="ts">
    import {
        Pause,
        Play,
        Repeat,
        Repeat1,
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
    import { LoopState } from './state.svelte'

    async function handleBackSkip() {
        if (appState.playlist.isEmpty()) {
            return
        }

        if (
            appState.player.position >= 5 /* seconds */ ||
            appState.playlist.previous.length === 0
        ) {
            await emit(SEEK_TRACK, {
                guildId: appState.guildId,
                position: 0,
                parallel: false
            })
        } else {
            let toAdd = appState.playlist.last() as Track
            await emit(QUEUE_TRACK, {
                guildId: appState.guildId,
                trackData: toAdd,
                looping: appState.player.loopState === LoopState.LoopTrack,
                prepend: true,
                overwrite: false,
                volume: appState.player.volume
            })
        }
    }

    async function cycleLoopState() {
        switch (appState.player.loopState) {
            case LoopState.None:
                appState.player.loopState = LoopState.LoopPlaylist
                break
            case LoopState.LoopPlaylist:
                appState.player.loopState = LoopState.LoopTrack
                // Update current track to loop
                await emit(LOOP_TRACK, {
                    guildId: appState.guildId,
                    parallel: false
                })
                break
            case LoopState.LoopTrack:
                appState.player.loopState = LoopState.None
                // Update current track to no longer loop
                await emit(LOOP_TRACK, {
                    guildId: appState.guildId,
                    parallel: false
                })
                break
            default:
                break
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
            {#if appState.player.playing}
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
                onclick={cycleLoopState}
            >
                {#if appState.player.loopState === LoopState.None}
                    <Repeat />
                {:else if appState.player.loopState === LoopState.LoopPlaylist}
                    <Repeat color={activeColor} />
                {:else}
                    <Repeat1 color={activeColor} />
                {/if}
            </button>
        </div>

        <!-- Empty padding space. Width should be equal to volume slider -->
        <TrackProgressBar
            player={appState.player}
            duration={appState.playlist.current()?.duration}
        />
    </div>

    <VolumeSlider player={appState.player} classes="pt-4" />
</div>
