<script lang="ts">
    import {
        TrackAction,
        UPDATE_TRACKS,
        type TrackActionPayload
    } from '$lib/events'
    import type { Player } from '$lib/state.svelte'
    import {
        appState,
        MUTE_SETTING,
        SETTINGS_FILENAME,
        VOLUME_SETTING
    } from '$lib/stores.svelte'
    import { Slider } from '@skeletonlabs/skeleton-svelte'
    import { emit } from '@tauri-apps/api/event'
    import { load } from '@tauri-apps/plugin-store'
    import { VolumeX, Volume2 } from 'lucide-svelte'
    import { onMount } from 'svelte'

    interface Props {
        player: Player
        uuid?: string
        classes?: string
    }

    let { player, uuid, classes }: Props = $props()

    async function onVolumeChange() {
        if (!uuid) {
            // The main player volume should be the only one with no UUID but we save
            // volume from appStore.mainPlayer directly just to be safe
            const store = await load(SETTINGS_FILENAME)
            store.set(VOLUME_SETTING, appState.player.volume)
        }

        if (player.mute) return

        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.ChangeVolume,
            volume: player.volume,
            parallel: uuid ? true : false,
            uuid
        } satisfies TrackActionPayload)
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

        await emit(UPDATE_TRACKS, {
            guildId: appState.guildId,
            action: TrackAction.ChangeVolume,
            volume,
            parallel: uuid ? true : false,
            uuid
        } satisfies TrackActionPayload)

        if (!uuid) {
            // Persist the main player's mute state
            const store = await load(SETTINGS_FILENAME)
            await store.set(MUTE_SETTING, appState.player.mute)
        }
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

    onMount(async () => {
        if (!uuid) {
            const store = await load(SETTINGS_FILENAME)
            appState.player.volume =
                (await store.get(VOLUME_SETTING)) ?? appState.player.volume
        }
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
