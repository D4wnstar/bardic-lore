<!--
 Original code by Lure5134 from this gist:
 https://gist.github.com/Lure5134/38001e338c95c830fb4725330f4ef048
-->

<script lang="ts" generics="T">
    // Import necessary Svelte utilities
    import { onMount, tick, type Snippet } from 'svelte'

    interface Props {
        /**
         * Array of items to render
         */
        items: Array<T>
        /**
         * Optional height of the viewport container
         */
        height?: string
        /**
         * Optional fixed height for each item
         */
        itemHeight?: number | undefined
        /**
         * Optional fixed width for each item
         */
        itemWidth?: number | undefined
        /**
         * Minimum gap between items
         */
        gap?: number | undefined
        /**
         * Snippet for rendering each item
         */
        children: Snippet<[T]>
        /**
         * Optional function to generate unique keys
         */
        getKey?: (item: T) => string | number
    }

    let {
        items,
        height = '100%',
        itemHeight = 200,
        itemWidth = 200,
        gap = 8,
        children,
        getKey
    }: Props = $props()

    // State variables for tracking visible items and layout
    let start = $state(0) // Index of first visible row
    let end = $state(0) // Index of last visible row
    let itemsPerRow = $state(1) // Number of items that fit in a row
    let containerWidth = $state(0) // Container width for calculating items per row

    // State for managing row heights and layout
    let height_map: Array<number> = $state([]) // Measured row heights
    let row_elements: HTMLCollectionOf<Element> = $state(null!)
    let viewport: HTMLElement = $state(null!)
    let contents: HTMLElement = $state(null!)
    let viewport_height = $state(0)
    let mounted: boolean = $state(false)

    // State for virtual padding calculations
    let top = $state(0) // Padding above visible items
    let bottom = $state(0) // Padding below visible items
    let average_height: number = $state(null!) // Average item height for estimation
    let left = $state(4) // Padding left of each row (to keep things centered)

    // Calculate rows from items based on items per row
    const rows = $derived.by(() => {
        const result: Array<Array<T>> = []
        for (let i = 0; i < items.length; i += itemsPerRow) {
            result.push(items.slice(i, i + itemsPerRow))
        }
        return result
    })

    // Derived state for visible rows with unique keys
    const visible = $derived.by(() => {
        const visibleRows = rows.slice(start, end)
        return visibleRows.map((row: T[], rowIndex: number) => {
            // Generate a unique ID for the entire row
            const rowStartIndex = (start + rowIndex) * itemsPerRow
            const rowId = row
                .map((item, i) => getKey?.(item) ?? rowStartIndex + i)
                .join(',')
            return {
                id: rowId,
                items: row
            }
        })
    })

    // Update items per row when container width changes
    function updateItemsPerRow() {
        const availableWidth = containerWidth
        itemsPerRow = Math.max(
            1,
            Math.floor(availableWidth / (itemWidth + gap))
        )
        // Add padding to the left to keep items centered
        const spaceLeft =
            availableWidth - itemWidth * itemsPerRow - gap * (itemsPerRow - 1)
        left = spaceLeft / 2
    }

    // Watch for container width changes
    $effect(() => {
        if (containerWidth) {
            updateItemsPerRow()
            refresh(items, viewport_height, itemHeight)
        }
    })

    // Whenever `items` changes, invalidate the current heightmap
    $effect(() => {
        if (mounted) {
            refresh(items, viewport_height, itemHeight) // Recalculate visible items
        }
    })

    // Refresh visible rows and measurements
    async function refresh(
        items: Array<T>,
        viewport_height: number,
        itemHeight?: number
    ) {
        // Current scroll position
        const { scrollTop } = viewport

        await tick() // Wait until the DOM is up to date

        let content_height = top - scrollTop
        let i = start

        // Calculate which items should be visible
        while (content_height < viewport_height && i < items.length) {
            let row = row_elements[i - start]

            // If row doesn't exist, render it
            if (!row) {
                end = i + 1
                await tick() // Wait for new row to render
                row = row_elements[i - start]
            }

            // Measure row height (use fixed height if provided)
            const row_height = (height_map[i] =
                itemHeight ?? (row as HTMLElement).offsetHeight)
            content_height += row_height
            i += 1
        }

        end = i // Update end index

        // Calculate padding for remaining rows
        const remaining = rows.length - end
        average_height = (top + content_height) / end

        if (end === 0) {
            average_height = 0
        }

        bottom = remaining * average_height
        height_map.length = items.length

        // Scroll only if we are outside the viewbox
        if (top >= bottom) {
            viewport.scrollTo(0, 0)
        }
    }

    // Handle scroll events to update visible items
    async function handle_scroll() {
        const { scrollTop } = viewport
        const old_start = start

        // Update height map for currently visible rows
        for (let v = 0; v < row_elements.length; v += 1) {
            height_map[start + v] =
                itemHeight ?? (row_elements[v] as HTMLElement).offsetHeight
        }

        let i = 0
        let y = 0

        // Find new start index based on scroll position
        while (i < items.length) {
            const row_height = height_map[i] || average_height
            if (y + row_height > scrollTop - viewport_height / 2) {
                start = i
                top = y
                break
            }
            y += row_height
            i += 1
        }

        // Find new end index based on viewport height
        while (i < items.length) {
            y += height_map[i] || average_height
            i += 1
            if (y > scrollTop + viewport_height * 1.5) break
        }

        end = i

        // Calculate average height from visible rows
        let visible_height = 0
        let visible_count = 0
        for (let j = start; j < end; j++) {
            if (height_map[j]) {
                visible_height += height_map[j]
                visible_count++
            }
        }
        average_height =
            visible_count > 0 ? visible_height / visible_count : itemHeight

        // Update padding for remaining rows
        const remaining = rows.length - end
        bottom = remaining * average_height

        // Only update height map for newly visible rows
        if (start < old_start) {
            await tick()
            for (let i = start; i < old_start; i++) {
                if (row_elements[i - start]) {
                    height_map[i] =
                        itemHeight ??
                        (row_elements[i - start] as HTMLElement).offsetHeight
                }
            }
        }
    }

    // Trigger initial refresh
    onMount(() => {
        // Get references to rendered row elements
        row_elements = contents.getElementsByTagName('virtual-list-row')
        mounted = true
    })
</script>

<virtual-list-viewport
    bind:this={viewport}
    bind:offsetHeight={viewport_height}
    bind:offsetWidth={containerWidth}
    onscroll={handle_scroll}
    style="height: {height}; padding: 4px 0px 0px {left}px"
>
    <virtual-list-contents
        bind:this={contents}
        style="padding-top: {top}px; padding-bottom: {bottom}px;"
    >
        {#each visible as row (row.id)}
            <virtual-list-row>
                {#each row.items as item}
                    <div
                        class="shrink-0"
                        style="width: {itemWidth}px; margin-right: {gap}px;"
                    >
                        {@render children?.(item)}
                    </div>
                {/each}
            </virtual-list-row>
        {/each}
    </virtual-list-contents>
</virtual-list-viewport>

<style>
    virtual-list-viewport {
        position: relative;
        overflow-y: auto;
        display: block;
    }

    virtual-list-contents {
        display: block;
    }

    virtual-list-row {
        display: flex;
        flex-wrap: wrap;
        padding-bottom: 8px;
    }
</style>
