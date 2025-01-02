<script lang="ts">
    import type { GuildSlug, VoiceChannelSlug } from '$lib/types'
    import { emit } from '@tauri-apps/api/event'

    interface Props {
        guild: GuildSlug
    }

    let { guild }: Props = $props()

    async function joinVoiceChannel(channel: VoiceChannelSlug) {
        console.log('Clicked', channel.name)
        await emit('join-voice-channel', { channel })
    }
</script>

<div
    class="preset-filled-surface-100-900 !bg-opacity-50 px-4 py-2 rounded-md border-[1px] border-primary-100-900"
>
    <h4 class="type-scale-5 grow text-secondary-700-300">
        <strong>{guild.name}</strong>
    </h4>
    <hr class="hr my-2" />
    <div class="space-y-1">
        {#each guild.voice_channels.toSorted( (a, b) => a.name.localeCompare(b.name) ) as channel}
            <button
                class="hover:bg-primary-100-900 p-1 w-full justify-start text-left"
                onclick={() => {
                    joinVoiceChannel(channel)
                }}
            >
                {channel.name}
            </button>
        {/each}
    </div>
</div>
