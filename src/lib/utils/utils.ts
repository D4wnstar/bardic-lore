import type { Player } from '$lib/state.svelte'
import { appState } from '$lib/stores.svelte'

/**
 * Find the parallel `PlayerState` associated with the track of the given path.
 * @param path The filepath to search by
 * @returns A `PlayerState`, if any was found
 */
export function getPlayerByUuid(uuid: string): Player | undefined {
    const maybePlayer = appState.parallelPlayers.find(
        ({ track }) => track.uuid === uuid
    )
    if (!maybePlayer) {
        console.warn(`Failed to find track with UUID ${uuid}`)
        return
    }
    return maybePlayer.player
}

/**
 * Converts a decimal RGB string into its hexadecimal form.
 * @param rgb A string in the format 'x y z' where x, y and z are decimal numbers
 * @returns A string of the format '#000000' in hex digits
 */
export function rgbToHex(rgb: string): string {
    const [r, g, b] = rgb.split(' ').map(Number)
    const toHex = (value: number) => value.toString(16).padStart(2, '0')
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`
}
