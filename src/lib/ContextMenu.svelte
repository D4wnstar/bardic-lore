<script lang="ts">
    import type { Icon } from 'lucide-svelte'
    import { onMount, onDestroy } from 'svelte'
    import { fade } from 'svelte/transition'

    interface Props {
        x: number
        y: number
        items: {
            Icon: typeof Icon
            label: string
            onclick: () => void
        }[]
        onclose: () => void
    }

    let { x, y, items, onclose }: Props = $props()

    let menu: HTMLElement | undefined = $state()

    function handleClickOutside(event: MouseEvent) {
        // Ignore clicks if menu isn't visible
        if (!menu || !x || !y) return

        // Ignore clicks inside the menu
        if (menu.contains(event.target as Node)) return

        onclose()
    }

    onMount(() => {
        // Use capture phase to ensure we handle clicks before other handlers
        document.addEventListener('click', handleClickOutside, {
            capture: true
        })
        document.addEventListener('contextmenu', handleClickOutside, {
            capture: true
        })
    })

    onDestroy(() => {
        document.removeEventListener('click', handleClickOutside, {
            capture: true
        })
        document.removeEventListener('contextmenu', handleClickOutside, {
            capture: true
        })
    })
</script>

{#if x && y}
    <div
        class="absolute z-50 bg-surface-100-900 p-1 rounded-md shadow-lg"
        style={`left: ${x + 1}px; top: ${y}px`}
        bind:this={menu}
        transition:fade={{ duration: 150 }}
    >
        {#each items as item}
            <button
                class="p-2 hover:bg-surface-200-800 rounded-md flex gap-2 w-full"
                onclick={() => {
                    item.onclick()
                    onclose()
                }}
            >
                <item.Icon />
                {item.label}
            </button>
        {/each}
    </div>
{/if}
