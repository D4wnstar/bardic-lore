<script lang="ts">
    import {
        type TrackActionPayload,
        UPDATE_TRACKS,
        TrackAction
    } from '$lib/events'
    import { LoopState } from '$lib/state.svelte'
    import { appState, type ParallelState } from '$lib/stores.svelte'
    import TrackProgressBar from '$lib/utils/TrackProgressBar.svelte'
    import { rgbToHex } from '$lib/utils/utils'
    import VolumeSlider from '$lib/utils/VolumeSlider.svelte'
    import { emit } from '@tauri-apps/api/event'
    import {
        Music,
        Pause,
        Play,
        Repeat,
        Repeat1,
        SkipBack,
        X
    } from 'lucide-svelte'

    interface Props {
        state: ParallelState
        parallel?: boolean
    }

    let { state, parallel }: Props = $props()

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )

    async function resumeParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Resume,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        } satisfies TrackActionPayload)
    }

    async function pauseParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Pause,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        } satisfies TrackActionPayload)
    }

    async function stopParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Stop,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        } satisfies TrackActionPayload)
    }

    async function skipBack() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Seek,
            parallel: parallel ?? false,
            uuid: state.track.uuid,
            position: 0
        } satisfies TrackActionPayload)
    }

    async function loopTrack() {
        if (parallel) {
            state.player.loopState =
                state.player.loopState === LoopState.None
                    ? LoopState.LoopTrack
                    : LoopState.None
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                action: TrackAction.Loop,
                parallel: parallel ?? false,
                uuid: state.track.uuid
            } satisfies TrackActionPayload)
        }
    }
</script>

<div class="w-full preset-filled-surface-100-900 !bg-opacity-50 rounded-md">
    <div class="flex">
        <!-- Becomes X icon on hover and allows removing track -->
        <div class="min-h-full min-w-16 flex items-center justify-center">
            <Music />
        </div>
        <div class="flex flex-col pb-1 pt-2 pr-4">
            <p class="line-clamp-1">{state.track.title}</p>
            <p class="opacity-50 line-clamp-1">{state.track.album}</p>
        </div>
    </div>
    <div class="flex mx-auto px-2 justify-center">
        <button class="btn-icon" onclick={skipBack}>
            <SkipBack />
        </button>
        {#if state.player.playing}
            <button class="btn-icon" onclick={pauseParallel}>
                <Pause />
            </button>
        {:else}
            <button class="btn-icon" onclick={resumeParallel}>
                <Play />
            </button>
        {/if}
        <button class="btn-icon" onclick={stopParallel}>
            <X />
        </button>
        <button class="btn-icon" onclick={loopTrack}>
            {#if state.player.loopState == LoopState.LoopTrack}
                <Repeat1 color={activeColor} />
            {:else}
                <Repeat />
            {/if}
        </button>
    </div>

    <VolumeSlider
        classes="pl-2 pr-4"
        player={state.player}
        uuid={parallel ? state.track.uuid : undefined}
    />

    <TrackProgressBar
        classes="pb-2"
        player={state.player}
        duration={state.track.duration}
        uuid={parallel ? state.track.uuid : undefined}
    />
</div>
