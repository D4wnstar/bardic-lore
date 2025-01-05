<script lang="ts">
    import type { Track } from '$lib/types'
    import { Music } from 'lucide-svelte'

    interface Props {
        track: Track
    }

    let { track }: Props = $props()
</script>

<div
    class="w-full h-16 preset-filled-surface-100-900 !bg-opacity-50 flex rounded-md"
>
    <div class="min-h-full min-w-16 flex items-center justify-center">
        <Music />
    </div>
    <div class="flex flex-col py-2 pr-4">
        <p class="line-clamp-1">{track.title}</p>
        <p class="opacity-50 line-clamp-1">{track.album}</p>
    </div>
</div>
