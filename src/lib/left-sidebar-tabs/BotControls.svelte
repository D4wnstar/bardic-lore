<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onDestroy, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
    import {
        BOT_TOKEN_SETTING,
        DISCORD_FILENAME,
        GUILDS_SETTING,
        SETTINGS_FILENAME,
        appState
    } from '$lib/stores.svelte'
    import type { GuildSlug, VoiceChannelSlug } from '$lib/types'
    import ServerBox from './ServerBox.svelte'
    import {
        JOIN_VOICE_CHANNEL,
        LEAVE_VOICE_CHANNEL,
        UPDATED_GUILDS
    } from '$lib/events'

    const toast: ToastContext = getContext('toast')

    let guilds: GuildSlug[] = $state([])
    let localGuild: GuildSlug = $state({
        id: 0,
        name: 'Offline Player',
        voice_channels: [{ id: 0, name: 'Offline', active: false }],
        offline: true
    })

    let botToken = $state('')
    let botConnected = $state(false)

    async function createClient() {
        await invoke('create_discord_client')
            .then(() => {
                toast.create({
                    title: 'Created client',
                    description:
                        'Successfully created client. Servers should refresh automatically in a moment',
                    type: 'success',
                    duration: 4000
                })
                botConnected = true
            })
            .catch((err) => {
                console.error(err)
                toast.create({
                    title: 'Error',
                    description: err,
                    type: 'error'
                })
            })
    }

    async function refreshServers(makeToast = false) {
        const store = await load(DISCORD_FILENAME, { autoSave: false })
        guilds = (await store.get<GuildSlug[]>(GUILDS_SETTING)) ?? []
        guilds.forEach((guild) => (guild.offline = false))

        // If no voice channel is active, activate the offline channel
        let localActiveState = true
        guilds.forEach((guild) =>
            guild.voice_channels.forEach((vchan) => {
                if (vchan.active) {
                    localActiveState = false
                    return
                }
            })
        )
        localGuild.voice_channels[0].active = localActiveState

        if (makeToast) {
            toast.create({
                description: 'Refreshed servers.',
                type: 'info'
            })
        }
    }

    async function isBotConnected() {
        await invoke<boolean>('is_bot_connected').then(
            (bool) => (botConnected = bool)
        )
    }

    async function getBotToken() {
        const store = await load(SETTINGS_FILENAME, { autoSave: false })
        botToken = (await store.get(BOT_TOKEN_SETTING)) ?? ''
    }

    async function updateBotToken() {
        const store = await load(SETTINGS_FILENAME, { autoSave: true })
        await store.set(BOT_TOKEN_SETTING, botToken)
    }

    async function handleChannelClick(
        guild: GuildSlug,
        channel: VoiceChannelSlug
    ) {
        if (guild.offline) {
            // Find the guild the bot is currently in by findind the active voice channel
            let activeGuild = guilds.find((guild) =>
                guild.voice_channels.find((ch) => ch.active)
            )
            if (!activeGuild) {
                return
            }
            await emit(LEAVE_VOICE_CHANNEL, { guildId: activeGuild.id })
            localGuild.voice_channels[0].active = true
        } else {
            await emit(JOIN_VOICE_CHANNEL, {
                guildId: guild.id,
                channelId: channel.id
            })
            localGuild.voice_channels[0].active = false
        }

        // Update the guild ID store
        appState.guildId = guild.id
        appState.offline = guild.offline

        // Update voice channels for the UI
        for (const currGuild of guilds) {
            for (const vchan of currGuild.voice_channels) {
                vchan.active =
                    currGuild.id === guild.id && vchan.id === channel.id
                        ? true
                        : false
            }
        }

        // Also update the store
        const store = await load(DISCORD_FILENAME, { autoSave: true })
        await store.set(GUILDS_SETTING, guilds)
    }

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        isBotConnected()
        refreshServers(false)
        getBotToken()
        unlisten = await listen<undefined>(UPDATED_GUILDS, () =>
            refreshServers(false)
        )
    })

    onDestroy(() => {
        if (unlisten) unlisten
    })
</script>

<div id="sources-sidebar" class="flex flex-col h-full min-h-0 pb-4 space-y-2">
    <h3 class="type-scale-7 heading-font-weight px-2 text-primary-900-100">
        Bot Controls
    </h3>
    <small class="small px-2 text-justify">
        You need a token for Bardic Lore to communicate to the bot. Copy yours
        below.
    </small>

    <div class="px-2"><strong>Bot token</strong></div>
    <input
        type="password"
        name="bot-token"
        id="bot-token"
        placeholder="Copy your token here..."
        class="input rounded-md"
        bind:value={botToken}
        oninput={updateBotToken}
    />

    <div class="flex w-full gap-2 justify-center py-2">
        {#if botConnected}
            <button class="btn preset-outlined-primary-400-600" disabled
                >Connected!
            </button>
        {:else}
            <button
                class="btn preset-outlined-primary-400-600"
                onclick={createClient}>Connect</button
            >
        {/if}
        <button
            class="btn preset-outlined-primary-400-600"
            onclick={() => refreshServers(true)}>Refresh servers</button
        >
    </div>

    <div class="overflow-auto space-y-2">
        <ServerBox guild={localGuild} onClick={handleChannelClick} />
        {#each guilds as guild}
            <ServerBox {guild} onClick={handleChannelClick} />
        {/each}
    </div>
</div>
