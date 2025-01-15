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
    <div class="h-screen w-screen">
        {@render children()}
    </div>
</ToastProvider>
