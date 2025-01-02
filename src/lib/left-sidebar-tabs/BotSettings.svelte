<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { type ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { getContext, onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { DISCORD_FILENAME, GUILDS_NAME } from '$lib/store'
    import type { GuildSlug } from '$lib/types'
    import ServerBox from './ServerBox.svelte'

    const toast: ToastContext = getContext('toast')

    let guilds: GuildSlug[] = $state([])

    async function createClient() {
        await invoke('create_discord_client')
            .catch((err) => {
                console.error(err)
                toast.create({
                    title: 'Error',
                    description: err,
                    type: 'error'
                })
            })
            .then(() => {
                toast.create({
                    title: 'Created client',
                    description: 'Successfully created client',
                    type: 'success'
                })
            })
    }

    async function refreshServers() {
        const store = await load(DISCORD_FILENAME, { autoSave: false })
        guilds = (await store.get<GuildSlug[]>(GUILDS_NAME)) ?? []
    }

    $inspect(guilds)

    onMount(refreshServers)
</script>

<div id="sources-sidebar" class="flex flex-col h-full min-h-0 pb-4 space-y-2">
    <h3 class="h3 text-center">Bot Settings</h3>
    <small class="small px-2 text-justify">
        These settings control the Discord bot.
    </small>

    <div class="flex w-full gap-2 justify-center py-2">
        <button class="btn preset-filled-primary-500" onclick={createClient}
            >Connect</button
        >
        <button class="btn preset-filled-primary-500" onclick={refreshServers}
            >Refresh servers</button
        >
    </div>

    {#each guilds as guild}
        <ServerBox {guild} />
    {/each}
</div>
