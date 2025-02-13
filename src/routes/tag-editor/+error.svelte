<script lang="ts">
    import { page } from '$app/state'
    import { ChevronLeft } from 'lucide-svelte'
</script>

<a href="/" class="flex"><ChevronLeft class="inline" />Back</a>
<div class="type-scale-5 grow p-2">{page.error?.message}</div>
