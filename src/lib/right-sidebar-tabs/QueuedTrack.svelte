<script lang="ts">
    import type { Track } from '$lib/types'
    import { getCover } from '$lib/utils/utils'
    import { Music } from 'lucide-svelte'
    import { onMount } from 'svelte'

    interface Props {
        track: Track
    }

    let { track }: Props = $props()

    let coverImage: string | undefined = $state()
    onMount(async () => {
        coverImage = await getCover('thumbnail', track.coverHash)
    })
</script>

<div
    class="w-full h-16 preset-filled-surface-100-900 !bg-opacity-50 flex rounded-md"
>
    <div class="min-h-full min-w-16 flex items-center justify-center">
        {#key coverImage}
            {#if coverImage}
                <img
                    src={coverImage}
                    alt="Album cover"
                    class="w-12 rounded-md"
                />
            {:else}
                <Music size="32" strokeWidth="1.75" />
            {/if}
        {/key}
    </div>
    <div class="flex flex-col justify-center py-2 pl-1 pr-4">
        <p
            class={{
                'line-clamp-1': track.album,
                'line-clamp-2': !track.album
            }}
        >
            {track.title}
        </p>
        {#if track.album}
            <p class="opacity-50 line-clamp-1">{track.album}</p>
        {/if}
    </div>
</div>
