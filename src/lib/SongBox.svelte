<script lang="ts">
    interface Props {
        title?: string
        tags?: string
    }

    let { title = 'Song Title', tags = 'Tags here' }: Props = $props()
</script>

<button
    class="card card-hover preset-filled-surface-100-900 !bg-opacity-50 flex max-w-60 flex-[12rem] flex-col items-center space-y-2 p-2 text-center border-[1px] border-transparent hover:border-primary-100-900"
>
    <h3 class="type-scale-5 text-primary-700-300">{title}</h3>
    <span>{tags}</span>
</button>
