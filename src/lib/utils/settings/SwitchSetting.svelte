<script lang="ts">
    import { Switch } from '@skeletonlabs/skeleton-svelte'

    interface Props {
        name: string
        description: string
        switchName: string
        checked: boolean
        onCheckedChange: (checked: boolean) => Promise<void>
    }

    let {
        name,
        description,
        switchName,
        checked = $bindable(),
        onCheckedChange
    }: Props = $props()
</script>

<div class="flex gap-4">
    <div class="grow space-y-1">
        <div>{name}</div>
        <div class="type-scale-1 text-surface-800-200">{description}</div>
    </div>
    <Switch
        name={switchName}
        bind:checked
        onCheckedChange={async (details) => onCheckedChange(details.checked)}
    />
</div>
