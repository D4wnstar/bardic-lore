import type { Track } from './types'

type TrackInternal = {
    track: Track
    singleUse: boolean
}

/**
 * A playlist abstraction to handle the state of the queue, including support
 * for manually adding one-time tracks, previously played tracks for backskips
 * and shuffling.
 */
export class Playlist {
    /**
     * This is a queue (use `push`, `shift` and `unshift`) that contains tracks
     * that are will be played in order.
     */
    #queue: TrackInternal[] = $state([])
    /**
     * This is a stack (use `push` and `pop`) that contains tracks that played
     * and ended.
     */
    #previous: TrackInternal[] = $state([])
    /**
     * This is a queue (use `push`, `shift` and `unshift`) that contains one-time
     * user determined tracks to play.
     */
    #priority: TrackInternal[] = $state([])

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
    }

    get queue() {
        return this.#queue.map((t) => t.track)
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
        if (this.#queue[0]) {
            return this.#queue[0].track
        }
    }

    /**
     * @returns The last played track, if any.
     */
    last() {
        if (this.#previous.length > 0) {
            return this.#previous[-1].track
        }
    }

    /**
     * Returns a list of all queued tracks, including priority ones.
     * @returns An object containing an array of queued tracks and an array
     * of priority tracks.
     */
    queued() {
        const priority = this.#priority.map((t) => t.track)
        const queued = this.#queue.map((t) => t.track)
        return { priority, queued }
    }

    /**
     * Advance the queue to the next track, prioritizing priority tracks and
     * moving the just ended track to the previous array.
     * @returns The track that just ended and the newly playing track
     */
    next() {
        // If there is a manually added track, prepend it to the queue
        // If not, use the next track in the queue
        // Also add the track that just ended into the previous ones
        if (this.#queue.length === 0 && this.#priority.length === 0) {
            return
        }

        // Single-use priority tracks should not be added to previously played
        let justEnded = this.#queue.shift() as TrackInternal
        if (justEnded && !justEnded.singleUse) {
            this.#previous.push(justEnded)
        }

        let nextTrack: TrackInternal | undefined
        if (this.#priority.length > 0) {
            nextTrack = this.#priority.shift() as TrackInternal
            this.#queue.unshift(nextTrack)
        }

        return { justEnded, nextTrack }
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
        if (this.#queue[0]) {
            let toOverwrite = this.#queue[0]
            this.#previous.push(toOverwrite)
            this.#queue[0] = { track, singleUse: false }
            return toOverwrite.track
        }
    }

    /**
     * Add the previous played track to the queue.
     * @returns The track that was added back, if any
     */
    backskip() {
        if (this.#previous.length > 0) {
            const toAdd = this.#previous.pop() as TrackInternal
            this.#queue.unshift(toAdd)
            return toAdd.track
        }
    }

    /**
     * Check if the track is empty by checking both the normal and priority queues.
     */
    isEmpty() {
        return this.#queue.length === 0 && this.#priority.length === 0
    }

    /**
     * @returns Whether the current track is priority or not. False if there are no tracks.
     */
    isCurrentPriority() {
        return this.#queue[0]?.singleUse ?? false
    }

    /**
     * Clear the playlist, removing everything.
     */
    clear() {
        this.#queue = []
        this.#previous = []
        this.#priority = []
    }

    /**
     * Loops the playlist by moving all previously played tracks to the queue.
     */
    loop() {
        this.#previous.forEach((t) => this.#queue.push(t))
        this.#previous = []
    }
}

export enum LoopState {
    None,
    LoopPlaylist,
    LoopTrack
}

export class Player {
    #playing = $state(false)
    position = $state(0)
    volume = $state(0.5)
    loopState = $state(LoopState.None)
    mute = $state(false)
    private timerId: number | undefined = $state()

    constructor(opts: {
        playing: boolean
        position: number
        volume: number
        loopState: LoopState
        mute: boolean
    }) {
        this.#playing = opts.playing
        this.position = opts.position
        this.volume = opts.volume
        this.loopState = opts.loopState
        this.mute = opts.mute
        this.timerId = undefined
    }

    private addTimer() {
        if (!this.timerId) {
            this.timerId = setInterval(() => {
                this.position += 1
            }, 1000)
        }
    }

    private removeTimer() {
        if (this.timerId) {
            clearInterval(this.timerId)
            this.timerId = undefined
        }
    }

    start() {
        this.#playing = true
        this.addTimer()
    }

    stop() {
        this.#playing = false
        this.removeTimer()
    }

    reset() {
        this.#playing = false
        this.removeTimer()
        this.position = 0
    }

    get playing() {
        return this.#playing
    }
}
