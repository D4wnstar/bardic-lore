import { cachedCoverImages, cachedCoverThumbnails } from '$lib/stores.svelte'
import type { CachedTrack } from '$lib/types'
import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
import { invoke } from '@tauri-apps/api/core'
import { BaseDirectory, readFile } from '@tauri-apps/plugin-fs'

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

/**
 * Gets the cover image for a given path. If it's already loaded, this will
 * just return the correct value from the store. Otherwise, it will read the file.
 * @param kind Whether to retrieve a cover or a thumbnail
 * @param hash The hash of the image, as found in the Track object
 * @returns The URL of the image
 */
export async function getCover(kind: 'cover' | 'thumbnail', hash?: string) {
    if (!hash) return

    // Check if the blob was already computed before
    const maybeCover =
        kind === 'cover'
            ? cachedCoverImages.get(hash)
            : cachedCoverThumbnails.get(hash)
    if (maybeCover) {
        return maybeCover
    }

    const path = kind === 'cover' ? `covers/${hash}` : `thumbnails/${hash}`
    const bytes = await readFile(path, {
        baseDir: BaseDirectory.AppCache
    })
    const url = URL.createObjectURL(new Blob([bytes], { type: 'image/webp' }))

    // After creating the blob for the first time, store it globally to be shared
    if (kind === 'cover') {
        cachedCoverImages.set(path, url)
    } else {
        cachedCoverThumbnails.set(path, url)
    }

    return url
}

export async function createDiscordClient(toast: ToastContext) {
    // Make sure to not attempt to create a second client
    let botConnected = await invoke<boolean>('is_bot_connected')
    if (botConnected) return true

    await invoke('create_discord_client')
        .then(() => {
            botConnected = true

            toast.create({
                title: 'Created client',
                description:
                    'Successfully created client. Servers should refresh in a moment.',
                type: 'success',
                duration: 10000
            })
        })
        .catch((err) => {
            console.error(err)
            toast.create({
                title: 'Error',
                description: err,
                type: 'error'
            })
        })

    return botConnected
}
