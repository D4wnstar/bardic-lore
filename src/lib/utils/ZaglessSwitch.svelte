<script lang="ts">
    // This is a clone of Skeleton V3's Switch component but without the Zag.js machine
    // It was causing problems where the page was being scrolled down despite the body being
    // overflow-hidden, but only when the object the Switch was in was originally rendered out
    // of focus (i.e. not visible on load due to being below overflow scroll). The problem was in
    // the Zag stuff which I don't have access to, so I just cloned the source and ripped it out

    import type { Snippet } from 'svelte'

    interface SwitchProps {
        /** Set a unique name for the switch input. */
        name: string
        /** Set the checked state. */
        checked?: boolean
        /** Set the disabled state. */
        disabled?: boolean
        /** Set the compact display mode. */
        compact?: boolean
        focused?: boolean

        // Root ---
        /** Set base classes for the root element. */
        base?: string
        /** Provide arbitrary classes to the root element. */
        classes?: string

        // State ---
        /** Set classes for the focus state. */
        stateFocused?: string

        // Control ---
        /** Set base classes for the control element. */
        controlBase?: string
        /** Set inactive state classes for the control element. */
        controlInactive?: string
        /** Set active state classes for the control element. */
        controlActive?: string
        /** Set disabled state classes for the control element. */
        controlDisabled?: string
        /** Set width classes for the control element. */
        controlWidth?: string
        /** Set height classes for the control element. */
        controlHeight?: string
        /** Set padding classes for the control element. */
        controlPadding?: string
        /** Set rounded classes for the control element. */
        controlRounded?: string
        /** Set hover classes for the control element. */
        controlHover?: string
        /** Provide arbitrary classes to the control element. */
        controlClasses?: string

        // Thumb ---
        /** Set base classes for the thumb element. */
        thumbBase?: string
        /** Set inactive classes for the thumb element. */
        thumbInactive?: string
        /** Set active classes for the thumb element. */
        thumbActive?: string
        /** Set rounded classes for the thumb element. */
        thumbRounded?: string
        /** Set animation X-axis translate classes for the thumb element. */
        thumbTranslateX?: string
        /** Set animation transition classes for the thumb element. */
        thumbTransition?: string
        /** Set animation easing classes for the thumb element. */
        thumbEase?: string
        /** Set animation duration classes for the thumb element. */
        thumbDuration?: string
        /** Provide arbitrary classes to the thumb element. */
        thumbClasses?: string

        // Label ---
        /** Set base classes for the label element. */
        labelBase?: string
        /** Provide arbitrary classes to the label element. */
        labelClasses?: string

        // Icons ---
        /** Set base classes for the inactive icon child. */
        iconInactiveBase?: string
        /** Set base classes for the active icon child. */
        iconActiveBase?: string

        // Snippets ---
        /** The default children snippet. */
        children?: Snippet
        /** The inactive state snippet. */
        inactiveChild?: Snippet
        /** The active state snippet. */
        activeChild?: Snippet

        // Callbacks
        onCheckedChange?: (
            ev: Event & { currentTarget: EventTarget & HTMLInputElement }
        ) => void
    }

    let {
        name = '',
        checked = $bindable(false),
        disabled = $bindable(false),
        compact = $bindable(false),
        focused = $bindable(false),
        // Root (Track)
        base = 'inline-flex items-center gap-4',
        classes = '',
        // State
        stateFocused = 'data-[focus-visible]:focused',
        // Control
        controlBase = 'cursor-pointer transition duration-200',
        controlInactive = 'preset-filled-surface-200-800',
        controlActive = 'preset-filled-primary-500',
        controlDisabled = 'opacity-50 cursor-not-allowed',
        controlWidth = 'w-10',
        controlHeight = 'h-6',
        controlPadding = 'p-0.5',
        controlRounded = 'rounded-full',
        controlHover = 'hover:brightness-90 dark:hover:brightness-110',
        controlClasses = '',
        // Thumb
        thumbBase = 'right-0 aspect-square h-full flex justify-center items-center text-right cursor-pointer',
        thumbInactive = 'preset-filled-surface-50-950',
        thumbActive = 'bg-surface-50 text-surface-contrast-50',
        thumbRounded = 'rounded-full',
        thumbTranslateX = 'translate-x-4 rtl:-translate-x-4',
        thumbTransition = 'transition',
        thumbEase = 'ease-in-out',
        thumbDuration = 'duration-200',
        thumbClasses = '',
        // Label
        labelBase = '',
        labelClasses = '',
        // Icons
        iconInactiveBase = 'pointer-events-none',
        iconActiveBase = 'pointer-events-none',
        // Snippets
        children,
        inactiveChild,
        activeChild,
        //
        onCheckedChange
    }: SwitchProps = $props()

    // Set Compact Mode
    if (compact) {
        controlBase = thumbBase
        // Removes the height class
        controlHeight = ''
        // Thumb inherits track styles
        thumbInactive = controlInactive
        thumbActive = controlActive
        // Remove X-axis translate
        thumbTranslateX = ''
        // Remove padding
        controlPadding = ''
    }

    const rxTrackState = $derived(checked ? controlActive : controlInactive)
    const rxThumbState = $derived(
        checked ? `${thumbActive} ${thumbTranslateX}` : thumbInactive
    )
    const rxDisabled = $derived(disabled ? controlDisabled : '')
    const rxFocused = $derived(focused ? stateFocused : '')
</script>

<!-- @component A control for toggling between checked states. -->

<label class="{base} {classes}" data-testid="switch">
    <!-- Input -->
    <input
        type="checkbox"
        class="hidden"
        onchange={(ev) => {
            checked = !checked
            if (onCheckedChange) onCheckedChange(ev)
        }}
        data-testid="switch-input"
    />
    <!-- Control -->
    <span
        class="{controlBase} {rxTrackState} {rxFocused} {controlWidth} {controlHeight} {controlPadding} {controlRounded} {controlHover} {rxDisabled}  {controlClasses}"
        data-testid="switch-control"
    >
        <!-- Thumb -->
        <span
            class="{thumbBase} {rxThumbState} {thumbRounded} {thumbTransition} {thumbEase} {thumbDuration} {thumbClasses}"
            data-testid="switch-thumb"
        >
            <!-- Icon: Inactive -->
            {#if !checked && inactiveChild}
                <span
                    class={iconInactiveBase}
                    data-testid="switch-icon-inactive"
                    >{@render inactiveChild()}</span
                >
            {/if}
            <!-- Icon: Active -->
            {#if checked && activeChild}
                <span class={iconActiveBase} data-testid="switch-icon-active"
                    >{@render activeChild()}</span
                >
            {/if}
        </span>
    </span>
    <!-- Label -->
    {#if children}
        <span class="{labelBase} {labelClasses}" data-testid="switch-label">
            {@render children()}
        </span>
    {/if}
</label>
