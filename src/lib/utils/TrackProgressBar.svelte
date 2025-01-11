<script lang="ts">
    import { SEEK_TRACK } from '$lib/events'
    import type { Player } from '$lib/state.svelte'
    import { appState } from '$lib/stores.svelte'
    import { Progress } from '@skeletonlabs/skeleton-svelte'
    import { emit } from '@tauri-apps/api/event'

    interface Props {
        player: Player
        uuid?: string
        duration?: number
        classes?: string
    }

    let { player, uuid, duration, classes }: Props = $props()

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

    async function onBarClick(
        e: MouseEvent & {
            currentTarget: EventTarget & HTMLButtonElement
        }
    ) {
        if (!duration) return

        const rect = e.currentTarget.getBoundingClientRect()
        const clickX = e.clientX - rect.left
        const progressWidth = rect.width
        const seekTo = Math.floor((clickX / progressWidth) * duration)
        await emit(SEEK_TRACK, {
            guildId: appState.guildId,
            position: seekTo,
            parallel: uuid ? true : false,
            uuid
        })
    }

    let fmtProgress = $derived(formatSeconds(player.position))
    let fmtDuration = $derived(duration ? formatSeconds(duration) : '0:00')
</script>

<div class="flex items-center gap-4 px-4 max-w-[550px] grow mx-auto {classes}">
    <p class="type-scale-2 opacity-70">{fmtProgress}</p>
    <button class="w-full" onclick={onBarClick} aria-label="player-bar">
        <Progress
            max={duration ?? 60}
            meterBg="bg-white hover:bg-primary-400-600"
            trackBg="bg-surface-200-800 hover:bg-surface-300-700"
            height="h-1"
            value={player.position}
        />
    </button>
    <p class="type-scale-2 opacity-70">
        {fmtDuration}
    </p>
</div>
