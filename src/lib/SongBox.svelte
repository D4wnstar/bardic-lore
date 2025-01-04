<script lang="ts">
    import { globalGuildId } from './stores.svelte'
    import type { Track } from './types'
    import { emit } from '@tauri-apps/api/event'

    interface Props {
        track: Track
    }

    let { track }: Props = $props()

    async function handleClick() {
        if (globalGuildId.id !== 0) {
            await emit('play-track', {
                guildId: `${globalGuildId.id}`,
                filepath: track.path
            })
        }
    }
</script>

<button
    class="card card-hover preset-filled-surface-100-900 !bg-opacity-50 flex max-w-60 flex-[12rem] flex-col items-center space-y-2 p-2 text-center border-[1px] border-transparent hover:border-primary-100-900"
    onclick={handleClick}
>
    <h3 class="type-scale-5 text-primary-700-300">{track.title}</h3>
    <span>{track.artist}</span>
</button>
