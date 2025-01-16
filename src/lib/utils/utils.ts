import type { CachedTrack } from '$lib/types'

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

/**
 * Permutes an array by putting `startTrack` at the beginning, it if it present.
 * This will maintain the order of `tracks` and shift everything accordingly.
 * This allocates a new array.
 * @param startTrack The track that needs to be at the start of the array
 * @param tracks The array of tracks
 * @returns The permuted array
 */
export function permuteTracks(startTrack: CachedTrack, tracks: CachedTrack[]) {
    let maybeIndex = tracks.findIndex((t) => t === startTrack)
    if (maybeIndex === -1) return

    let firstBlock = tracks.slice(maybeIndex)
    let secondBlock = tracks.slice(0, maybeIndex)
    return [...firstBlock, ...secondBlock]
}
