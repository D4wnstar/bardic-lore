import type { Track } from '$lib/types'
import { SvelteSet } from 'svelte/reactivity'
import { TrackSet } from './trackset.svelte'

type TrackInternal = {
    track: Track
    singleUse: boolean
}

export enum SortMethod {
    Alphabetical
}

export enum SortOrder {
    Ascending,
    Descending
}

/**
 * A playlist abstraction to handle the state of the queue, including support
 * for manually adding one-time tracks, previously played tracks for backskips
 * and shuffling.
 */
export class Playlist {
    /**
     * This is a queue that contains tracks that will be played in order.
     * We mostly avoid shift operations by keeping track of removed indices.
     * The first track in the array is the next to be played.
     */
    #queue: TrackInternal[] = $state([])
    #removedQueueIndices = new SvelteSet<number>()
    #queueStartIndex = 0

    /**
     * This is a stack (use `push` and `pop`) that contains tracks that played
     * and ended. The last track in the array is the most recently played one.
     */
    #previous: TrackInternal[] = $state([])

    /**
     * This is a stack (use `push` and `pop`) that contains tracks that should
     * be played before anything in the queue and are single use. It follows a
     * FIFO approach, i.e. the last track in the array is the one the next to
     * be played.
     */
    #priority: TrackInternal[] = $state([])

    /**
     * Compact arrays by actually removing marked indices when they grow too large
     * This threshold can be tuned based on performance characteristics
     */
    private static COMPACT_THRESHOLD = 1000

    /**
     * Get internal state for testing purposes
     * @internal
     */
    getInternalState() {
        return {
            removedQueueIndices: new Set(this.#removedQueueIndices),
            queueStartIndex: this.#queueStartIndex,
            queueLength: this.#queue.length
        }
    }

    /**
     * Set compact threshold for testing purposes
     * @internal
     */
    static setCompactThreshold(threshold: number) {
        this.COMPACT_THRESHOLD = threshold
    }

    constructor(queue: Track[], previous: Track[], priority: Track[]) {
        this.#queue = queue.map((track) => {
            return { track, singleUse: false }
        })
        this.#previous = previous.map((track) => {
            return { track, singleUse: false }
        })
        this.#priority = priority.map((track) => {
            return { track, singleUse: true }
        })
        // Initialize tracking state
        this.#removedQueueIndices = new SvelteSet()
        this.#queueStartIndex = 0
    }

    private compactQueueIfNeeded(force?: boolean) {
        // Compact queue if we have too many removed indices
        if (
            this.#removedQueueIndices.size > Playlist.COMPACT_THRESHOLD ||
            force
        ) {
            this.#queue = this.#queue.filter(
                (_, i) => !this.#removedQueueIndices.has(i)
            )
            this.#removedQueueIndices.clear()
            this.#queueStartIndex = 0
        }
    }

    get queue() {
        return this.#queue
            .filter((_, i) => !this.#removedQueueIndices.has(i))
            .map((t) => t.track)
    }

    get previous() {
        return this.#previous.map((t) => t.track)
    }

    get priority() {
        return this.#priority.map((t) => t.track)
    }

    /**
     * @returns The current track in the queue, if any.
     */
    current() {
        // Find first non-removed track
        for (let i = this.#queueStartIndex; i < this.#queue.length; i++) {
            if (!this.#removedQueueIndices.has(i)) {
                return this.#queue[i].track
            }
        }
    }

    /**
     * @returns The last played track, if any.
     */
    last() {
        return this.#previous.at(-1)?.track
    }

    /**
     * Returns a list of all queued tracks, including priority ones.
     * @returns An object containing an array of queued tracks and an array
     * of priority tracks.
     */
    queued() {
        // TODO: This method is no longer really useful, maybe remove it?
        return { priority: this.priority, queued: this.queue }
    }

    /**
     * Returns a list of all of the tracks that make up the playlist.
     * @returns An array of `Track`s
     */
    tracks() {
        const previous = this.#previous.map((t) => t.track)
        const queued = this.#queue
            .filter((t, i) => !this.#removedQueueIndices.has(i) && !t.singleUse)
            .map((t) => t.track)
        return new TrackSet([...previous, ...queued])
    }

    /**
     * Add a track to the queue.
     * @param track The track to enqueue
     */
    enqueue(track: Track) {
        this.#queue.push({ track, singleUse: false })
    }

    /**
     * Add a track to the front of the queue.
     * @param track The track to enqueue
     */
    enqueueFront(track: Track) {
        // Compact the playlist so we can unshift safely
        this.compactQueueIfNeeded(true)
        this.#queue.unshift({ track, singleUse: false })
    }

    /**
     * Add a track to the priority, single-use queue.
     * @param track The track to enqueue
     */
    enqueuePriority(track: Track) {
        this.#priority.push({ track, singleUse: true })
    }

    /**
     * Overwrite the current track, if any, with the given track.
     * Will push the overwritten track to the previously played ones.
     * Will do nothing if there is no current track.
     * @param track The track to enqueue
     * @returns The overwritten track, if any
     */
    overwriteCurrent(track: Track) {
        // Find first non-removed track
        for (let i = this.#queueStartIndex; i < this.#queue.length; i++) {
            if (!this.#removedQueueIndices.has(i)) {
                let toOverwrite = this.#queue[i]
                this.#previous.push(toOverwrite)
                this.#queue[i] = { track, singleUse: false }
                return toOverwrite.track
            }
        }
    }

    /**
     * Advance the queue to the next track, prioritizing priority tracks and
     * moving the just ended track to the previous array.
     * @returns The track that just ended and the newly playing track
     */
    next() {
        // Find first non-removed track in queue
        let currentQueueIndex = -1
        for (let i = this.#queueStartIndex; i < this.#queue.length; i++) {
            if (!this.#removedQueueIndices.has(i)) {
                currentQueueIndex = i
                break
            }
        }

        let currentPriorityTrack = this.#priority.pop()

        if (currentQueueIndex === -1 && !currentPriorityTrack) {
            return
        }

        // Mark current track as removed and add to previous if not single use
        let justEnded: TrackInternal | undefined
        if (currentQueueIndex !== -1) {
            justEnded = this.#queue[currentQueueIndex]
            this.#removedQueueIndices.add(currentQueueIndex)
            if (!justEnded.singleUse) {
                this.#previous.push(justEnded)
            }
        }

        let nextTrack: TrackInternal | undefined
        if (currentPriorityTrack) {
            // Use priority track if one was found
            // Compact the playlist so we can unshift safely
            this.compactQueueIfNeeded(true)
            this.#queue.unshift(currentPriorityTrack)
            nextTrack = currentPriorityTrack
            // TODO: This unshift can probably be removed in some way
        } else if (currentQueueIndex !== -1) {
            // Find next non-removed track in queue
            for (let i = currentQueueIndex + 1; i < this.#queue.length; i++) {
                if (!this.#removedQueueIndices.has(i)) {
                    nextTrack = this.#queue[i]
                    break
                }
            }
        }

        this.compactQueueIfNeeded()

        return justEnded && nextTrack
            ? { justEnded: justEnded.track, nextTrack: nextTrack.track }
            : undefined
    }

    /**
     * Pops the previous played track.
     * @returns The popped track, if any
     */
    popPrevious() {
        return this.#previous.pop()?.track
    }

    /**
     * Check if the queue is empty by checking both the normal and priority queues.
     */
    isEmpty() {
        // Check if there are any non-removed tracks
        const hasQueueTracks = this.#queue.some(
            (_, i) => !this.#removedQueueIndices.has(i)
        )
        const hasPriorityTracks = this.#priority.length > 0
        return !hasQueueTracks && !hasPriorityTracks
    }

    /**
     * @returns Whether the current track is priority or not. False if there are no tracks.
     */
    isCurrentPriority() {
        // Find first non-removed track
        for (let i = this.#queueStartIndex; i < this.#queue.length; i++) {
            if (!this.#removedQueueIndices.has(i)) {
                return this.#queue[i]?.singleUse ?? false
            }
        }
        return false
    }

    /**
     * Clear the playlist, removing everything.
     */
    clear() {
        this.#queue = []
        this.#previous = []
        this.#priority = []
        this.#removedQueueIndices.clear()
        this.#queueStartIndex = 0
    }

    /**
     * Sorts the queue so that the tracks follow the same order as the one given by the `uuids` array.
     * This only sorts the queue, not the whole playlist. This means that previously played tracks
     * are not sorted and the playlist is not reset. To reset and sort the entire playlist, use
     * `sortByMethod`.
     * @param uuids The array of UUIDs to sort by
     * @param opts Whether to skip the first (and current) track
     * If this is true, the given UUIDs should NOT contain the UUID of the first track
     * @returns The sorted array of tracks
     */
    sortAsUuids(uuids: string[], opts = { skipFirst: false }) {
        // Compact the queue since we're resetting it anyway
        this.compactQueueIfNeeded(true)

        // Get current track if needed
        let currentTrack: TrackInternal | undefined
        if (opts.skipFirst) {
            currentTrack = this.#queue.shift()
        }

        let tracks = this.#queue

        // Sort tracks by UUID order
        const sortedTracks = uuids
            .map((uuid) => {
                const track = tracks.find((t) => t.track.uuid === uuid)
                if (!track) {
                    console.warn(`Failed to find track with UUID ${uuid}`)
                    return null
                }
                return track
            })
            .filter((track): track is TrackInternal => track !== null)

        // Rebuild queue with sorted tracks
        if (opts.skipFirst && currentTrack) {
            this.#queue = [currentTrack, ...sortedTracks]
        } else {
            this.#queue = sortedTracks
        }
    }

    /**
     * Reset the playlist by moving all previous tracks into the queue, then sort
     * the queue by the given method, leaving the current track untouched.
     * @param method The method to sort with
     * @param order The order to sort by
     */
    sortByMethod(method: SortMethod, order: SortOrder) {
        if (this.tracks().size < 2) return

        // Compact the queue since we're resetting it anyway
        this.compactQueueIfNeeded(true)

        // Get current track if exists
        let currentTrack: TrackInternal | undefined = this.#queue.shift()
        let tracks = this.#queue

        let sortedTracks = [...this.#previous, ...tracks]
        this.#previous = []

        // Sort tracks
        switch (method) {
            case SortMethod.Alphabetical:
                if (order === SortOrder.Ascending) {
                    sortedTracks = sortedTracks.sort((a, b) => {
                        const titleA = a.track.title ?? a.track.filename
                        const titleB = b.track.title ?? b.track.filename
                        return titleA.localeCompare(titleB)
                    })
                } else {
                    sortedTracks = sortedTracks.sort((a, b) => {
                        const titleA = a.track.title ?? a.track.filename
                        const titleB = b.track.title ?? b.track.filename
                        return titleB.localeCompare(titleA)
                    })
                }
                break
        }

        // Rebuild queue with current track at start if it exists
        this.#queue = currentTrack
            ? [currentTrack, ...sortedTracks]
            : sortedTracks
    }
}
