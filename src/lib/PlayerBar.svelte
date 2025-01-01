<script>
    import { Play, Repeat, Shuffle, SkipBack, SkipForward } from 'lucide-svelte'
    import { invoke } from '@tauri-apps/api/core'
</script>

<div class="preset-outlined-surface-500 mt-2 h-24 rounded-md p-2 flex-none">
    <div class="flex justify-center gap-2">
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><Shuffle /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><SkipBack /></button
        >
        <button
            class="btn-icon rounded-none hover:preset-filled-surface-500"
            onclick={() => invoke('play')}
        >
            <Play color="#2161b8" size="32" /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><SkipForward /></button
        >
        <button class="btn-icon rounded-none hover:preset-filled-surface-500"
            ><Repeat /></button
        >
    </div>
</div>
