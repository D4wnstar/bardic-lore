<script lang="ts">
    import '../app.css'
    import { type Snippet } from 'svelte'
    import { ToastProvider } from '@skeletonlabs/skeleton-svelte'

    let { children }: { children: Snippet } = $props()
</script>

<svelte:head>
    <title>Bardic Lore</title>
</svelte:head>

<ToastProvider>
    <!-- Prevent drag-to-select on the whole UI (except modals) -->
    <div class="h-screen w-screen select-none">
        {@render children()}
    </div>
</ToastProvider>
