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
    import {
        appState,
        LOOP_SETTING,
        SETTINGS_FILENAME,
        SHUFFLE_SETTING
    } from './stores.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        QUEUE_TRACK,
        QueueMethod,
        TrackAction,
        UPDATE_TRACKS,
        type QueueTrackPayload,
        type TrackActionPayload
    } from './events'
    import type { Track } from './types'
    import VolumeSlider from './utils/VolumeSlider.svelte'
    import TrackProgressBar from './utils/TrackProgressBar.svelte'
    import { rgbToHex } from './utils/utils'
    import { LoopState, SortMethod, SortOrder } from './state.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { invoke } from '@tauri-apps/api/core'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext } from 'svelte'

    const toast: ToastContext = getContext('toast')
    function errorToast(description: string) {
        toast.create({
            title: 'Error',
            description
        })
    }

    async function handlePlay() {
        const args = {
            action: TrackAction.Resume,
            parallel: false
        }

        if (!appState.offline) {
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                ...args
            } satisfies TrackActionPayload)
        } else {
            await invoke('queue_action', args)
        }
    }

    async function handlePause() {
        const args = {
            action: TrackAction.Pause,
            parallel: false
        }

        if (!appState.offline) {
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                ...args
            } satisfies TrackActionPayload)
        } else {
            await invoke('queue_action', args)
        }
    }

    async function handleSkip() {
        const args = {
            action: TrackAction.Skip,
            parallel: false
        }

        if (!appState.offline) {
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                ...args
            } satisfies TrackActionPayload)
        } else {
            await invoke('queue_action', args).catch((e) => errorToast(e))
        }
    }

    async function handleBackSkip() {
        if (appState.playlist.isEmpty()) {
            return
        }

        if (
            appState.player.position >= 5 /* seconds */ ||
            appState.playlist.previous.length === 0
        ) {
            // If the track has been playing for a while, reset the position to zero
            const args = {
                action: TrackAction.Seek,
                position: 0,
                parallel: false
            }

            if (!appState.offline) {
                await emit(UPDATE_TRACKS, {
                    guildId: appState.guildId,
                    ...args
                } satisfies TrackActionPayload)
            } else {
                await invoke('queue_action', args).catch((e) => errorToast(e))
            }
        } else {
            // If the track just started, go back to the previous one
            let toAdd = appState.playlist.last() as Track
            const args = {
                trackData: toAdd,
                queueMethod: QueueMethod.Backskip,
                looping: appState.player.loopState === LoopState.LoopTrack,
                volume: appState.player.volume
            }

            if (!appState.offline) {
                await emit(QUEUE_TRACK, {
                    guildId: appState.guildId,
                    ...args
                } satisfies QueueTrackPayload)
            } else {
                // Prepending is currently not supported by rodio
                const args = {
                    action: TrackAction.Seek,
                    position: 0,
                    parallel: false
                }
                await invoke('queue_action', args).catch((e) => errorToast(e))
            }
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
                await emit(UPDATE_TRACKS, {
                    guildId: appState.guildId,
                    action: TrackAction.Loop,
                    parallel: false
                } satisfies TrackActionPayload)
                break
            case LoopState.LoopTrack:
                appState.player.loopState = LoopState.None
                // Update current track to no longer loop
                await emit(UPDATE_TRACKS, {
                    guildId: appState.guildId,
                    action: TrackAction.Loop,
                    parallel: false
                } satisfies TrackActionPayload)
                break
            default:
                break
        }

        const store = await load(SETTINGS_FILENAME)
        await store.set(LOOP_SETTING, appState.player.loopState)
    }

    async function handleShuffleClick() {
        appState.player.shuffle = !appState.player.shuffle
        if (appState.player.shuffle) {
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                action: TrackAction.Shuffle,
                parallel: false
            } satisfies TrackActionPayload)
        } else {
            appState.playlist.sortByMethod(
                SortMethod.Alphabetical,
                SortOrder.Ascending
            )
            const sortUuids = appState.playlist.queue.map((t) => t.uuid)
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                action: TrackAction.Sort,
                parallel: false,
                sortUuids
            } satisfies TrackActionPayload)
        }

        const store = await load(SETTINGS_FILENAME)
        await store.set(SHUFFLE_SETTING, appState.player.shuffle)
    }

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )
</script>

<div
    class="border-t-[1px] border-surface-900 mt-2 h-24 p-2 flex-none flex items-center"
>
    <div class="2xl:min-w-[200px]"><!-- Padding --></div>
    <div class="grow">
        <div class="flex justify-center gap-2 mb-2 mt-2">
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-100-900"
                onclick={handleShuffleClick}
                ><Shuffle
                    color={appState.player.shuffle ? activeColor : '#ffffff'}
                /></button
            >
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-100-900"
                onclick={handleBackSkip}><SkipBack /></button
            >
            {#if appState.player.playing}
                <button
                    class="btn-icon rounded-none preset-filled-primary-100-900"
                    onclick={handlePause}><Pause /></button
                >
            {:else}
                <button
                    class="btn-icon rounded-none preset-filled-primary-100-900"
                    onclick={handlePlay}><Play /></button
                >
            {/if}
            <button
                class="btn-icon rounded-none hover:preset-filled-surface-100-900"
                onclick={handleSkip}><SkipForward /></button
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
