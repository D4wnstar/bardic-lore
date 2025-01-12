<script lang="ts">
    import type { GuildSlug } from '$lib/types'
    import { rgbToHex } from '$lib/utils/utils'
    import { Volume2, VolumeOff } from 'lucide-svelte'

    interface Props {
        guild: GuildSlug
        onClick: Function
    }

    let { guild, onClick }: Props = $props()

    const inactiveColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue('--color-surface-400')
    )
    const activeColor = rgbToHex(
        getComputedStyle(document.body).getPropertyValue(
            '--color-secondary-200'
        )
    )
</script>

<div
    class="preset-filled-surface-100-900 !bg-opacity-50 px-4 pt-2 pb-4 rounded-md border-[1px] border-primary-100-900 !border-opacity-70"
>
    <h4 class="type-scale-5 grow text-secondary-700-300">
        <strong>{guild.name}</strong>
    </h4>
    <hr class="hr my-2" />
    <div class="space-y-1">
        {#each guild.voice_channels.toSorted( (a, b) => a.name.localeCompare(b.name) ) as channel}
            <button
                class={[
                    'p-1 w-full items-center text-left flex gap-2 !bg-opacity-50 rounded-md',
                    !channel.active && 'hover:bg-primary-100-900',
                    channel.active && 'bg-primary-200-800 px-2'
                ]}
                onclick={() => {
                    onClick(guild, channel)
                }}
            >
                {#if !channel.active}
                    <VolumeOff color={inactiveColor} opacity="0.5" />
                {:else}
                    <Volume2 color={activeColor} />
                {/if}
                {channel.name}
            </button>
        {/each}
    </div>
</div>
