<script lang="ts">
    import { CHANGE_VOLUME } from '$lib/events'
    import { appState, type PlayerState } from '$lib/stores.svelte'
    import { Slider } from '@skeletonlabs/skeleton-svelte'
    import { emit } from '@tauri-apps/api/event'
    import { VolumeX, Volume2 } from 'lucide-svelte'

    interface Props {
        player: PlayerState
        uuid?: string
        classes?: string
    }

    let { player, uuid, classes }: Props = $props()

    async function onVolumeChange() {
        if (player.mute) return

        await emit(CHANGE_VOLUME, {
            guildId: appState.guildId,
            volume: player.volume,
            parallel: uuid ? true : false,
            uuid
        })
    }

    async function onMuteClick() {
        let volume: number
        if (player.mute) {
            volume = player.volume
            player.mute = false
        } else {
            volume = 0
            player.mute = true
        }

        await emit(CHANGE_VOLUME, {
            guildId: appState.guildId,
            volume,
            parallel: uuid ? true : false,
            uuid
        })
    }

    let value = $state([player.volume * 100])

    // Horrible hack to keep player volume and slider value synced.
    // These run on different scales (0 to 1.0 and 0 to 100 respectively)
    // and so I can't bind to player.volume directly. But I also can't
    // bind to $derived state, so I can't just have value be
    // "$derived([player.volume * 100])". This double $effect is
    // essentially a hacky two-way bind with a rescale in the middle
    $effect(() => {
        player.volume = value[0] / 100
    })
    $effect(() => {
        value[0] = player.volume * 100
    })
</script>

<div class="min-w-[200px] gap-x-3 flex {classes}">
    <button
        class="btn-icon rounded-none hover:preset-filled-surface-100-900"
        onclick={onMuteClick}
    >
        {#if player.mute}
            <VolumeX />
        {:else}
            <Volume2 />
        {/if}
    </button>
    <Slider
        classes="pt-[16px]"
        thumbCursor="cursor-ew-resize"
        height="h-1"
        bind:value
        onValueChangeEnd={onVolumeChange}
    />
</div>
