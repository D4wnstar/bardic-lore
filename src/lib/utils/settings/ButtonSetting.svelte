<script lang="ts">
    interface Props {
        name: string
        description: string
        buttonName: string
        onClick: () => Promise<void>
    }

    let { name, description, buttonName, onClick }: Props = $props()
</script>

<div class="flex gap-4 items-center">
    <div class="grow space-y-1">
        <header>{name}</header>
        <p class="type-scale-1 text-surface-800-200">{description}</p>
    </div>
    <button
        class="btn preset-outlined-primary-400-600"
        onclick={() => onClick()}>{buttonName}</button
    >
</div>
