<script lang="ts">
    import {
        type TrackActionPayload,
        UPDATE_TRACKS,
        TrackAction
    } from '$lib/events'
    import { LoopState, type ParallelState } from '$lib/state.svelte'
    import { appState } from '$lib/stores.svelte'
    import TrackProgressBar from '$lib/utils/TrackProgressBar.svelte'
    import { getCover, rgbToHex } from '$lib/utils/utils'
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
    import { onMount } from 'svelte'

    interface Props {
        parallelState: ParallelState
        parallel?: boolean
    }

    let { parallelState, parallel }: Props = $props()

    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-primary-400')
    )

    async function resumeParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Resume,
            parallel: parallel ?? false,
            uuid: parallelState.track.uuid
        } satisfies TrackActionPayload)
    }

    async function pauseParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Pause,
            parallel: parallel ?? false,
            uuid: parallelState.track.uuid
        } satisfies TrackActionPayload)
    }

    async function stopParallel() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Stop,
            parallel: parallel ?? false,
            uuid: parallelState.track.uuid
        } satisfies TrackActionPayload)
    }

    async function skipBack() {
        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.Seek,
            parallel: parallel ?? false,
            uuid: parallelState.track.uuid,
            position: 0
        } satisfies TrackActionPayload)
    }

    async function loopTrack() {
        if (parallel) {
            parallelState.player.loopState =
                parallelState.player.loopState === LoopState.None
                    ? LoopState.LoopTrack
                    : LoopState.None
            await emit(UPDATE_TRACKS, {
                guildId: appState.guildId,
                action: TrackAction.Loop,
                parallel: parallel ?? false,
                uuid: parallelState.track.uuid
            } satisfies TrackActionPayload)
        }
    }

    let coverImage: string | undefined = $state()
    onMount(async () => {
        coverImage = await getCover('thumbnail', parallelState.track.cover_hash)
    })
</script>

<div class="w-full preset-filled-surface-100-900 !bg-opacity-50 rounded-md">
    <div class="flex">
        <!-- Becomes X icon on hover and allows removing track -->
        <div class="min-h-full min-w-16 flex items-center justify-center">
            {#if coverImage}
                <img
                    src={coverImage}
                    alt="Album cover"
                    class="w-12 rounded-md"
                />
            {:else}
                <Music size="32" strokeWidth="1.75" />
            {/if}
        </div>
        <div class="flex flex-col pb-1 pt-2 pl-1 pr-4">
            <p class="line-clamp-1">{parallelState.track.title}</p>
            <p class="opacity-50 line-clamp-1">{parallelState.track.album}</p>
        </div>
    </div>
    <div class="flex mx-auto px-2 justify-center">
        <button class="btn-icon" onclick={skipBack}>
            <SkipBack />
        </button>
        {#if parallelState.player.playing}
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
            {#if parallelState.player.loopState == LoopState.LoopTrack}
                <Repeat1 color={activeColor} />
            {:else}
                <Repeat />
            {/if}
        </button>
    </div>

    <VolumeSlider
        classes="pl-2 pr-4"
        player={parallelState.player}
        uuid={parallel ? parallelState.track.uuid : undefined}
    />

    <TrackProgressBar
        classes="pb-2"
        player={parallelState.player}
        duration={parallelState.track.duration}
        uuid={parallel ? parallelState.track.uuid : undefined}
    />
</div>
