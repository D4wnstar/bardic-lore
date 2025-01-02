<script lang="ts">
    import type { GuildSlug } from '$lib/types'

    interface Props {
        guild: GuildSlug
    }

    let { guild }: Props = $props()
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
            >
                {channel.name}
            </button>
        {/each}
    </div>
</div>
