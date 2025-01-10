<script lang="ts">
    import {
        LOOP_TRACK,
        PAUSE_PLAYBACK,
        RESUME_PLAYBACK,
        SEEK_TRACK,
        STOP_TRACK
    } from '$lib/events'
    import { appState, type ParallelState } from '$lib/stores.svelte'
    import TrackProgressBar from '$lib/utils/TrackProgressBar.svelte'
    import { rgbToHex } from '$lib/utils/utils'
    import VolumeSlider from '$lib/utils/VolumeSlider.svelte'
    import { emit } from '@tauri-apps/api/event'
    import { Music, Pause, Play, Repeat, SkipBack, X } from 'lucide-svelte'

    interface Props {
        state: ParallelState
        parallel?: boolean
    }

    let { state, parallel }: Props = $props()

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )

    async function resumeParallel() {
        await emit(RESUME_PLAYBACK, {
            guildId: appState.guildId,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        })
    }

    async function pauseParallel() {
        await emit(PAUSE_PLAYBACK, {
            guildId: appState.guildId,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        })
    }

    async function stopParallel() {
        await emit(STOP_TRACK, {
            guildId: appState.guildId,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        })
    }

    async function skipBack() {
        await emit(SEEK_TRACK, {
            guildId: appState.guildId,
            parallel: parallel ?? false,
            uuid: state.track.uuid,
            position: 0
        })
    }

    async function loopTrack() {
        state.player.looping = !state.player.looping
        await emit(LOOP_TRACK, {
            guildId: appState.guildId,
            parallel: parallel ?? false,
            uuid: state.track.uuid
        })
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
            <Repeat color={state.player.looping ? activeColor : '#ffffff'} />
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
