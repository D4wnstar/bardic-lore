<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onDestroy, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { listen, type UnlistenFn } from '@tauri-apps/api/event'
    import {
        BOT_TOKEN_SETTING,
        DISCORD_FILENAME,
        GUILDS_SETTING,
        SETTINGS_FILENAME
    } from '$lib/store'
    import type { GuildSlug } from '$lib/types'
    import ServerBox from './ServerBox.svelte'

    const toast: ToastContext = getContext('toast')

    let guilds: GuildSlug[] = $state([])
    let botToken = $state('')
    let localGuild: GuildSlug = {
        id: 0,
        name: 'Offline Player',
        voice_channels: [{ id: 0, name: 'Offline' }]
    }
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
        if (makeToast) {
            toast.create({
                description: 'Refreshed servers',
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

    let unlisten: UnlistenFn | undefined
    onMount(async () => {
        isBotConnected()
        refreshServers(false)
        getBotToken()
        unlisten = await listen<undefined>('updated-guilds', () =>
            refreshServers(false)
        )
    })

    onDestroy(() => {
        if (unlisten) unlisten
    })
</script>

<div id="sources-sidebar" class="flex flex-col h-full min-h-0 pb-4 space-y-2">
    <h3 class="h3 text-center">Bot Settings</h3>
    <small class="small px-2 text-justify">
        These settings control the Discord bot.
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
            <button
                class="btn preset-outlined-primary-400-600"
                onclick={createClient}
                disabled
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

    <ServerBox guild={localGuild} />
    {#each guilds as guild}
        <ServerBox {guild} />
    {/each}
</div>
