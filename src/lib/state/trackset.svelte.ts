import type { CachedTrack } from '$lib/types'

/**
 * A hacky """`Set`""" to keep `Track`s unique.
 */
export class TrackSet {
    #tracks: CachedTrack[] = $state([])

    constructor(owners: CachedTrack[]) {
        this.#tracks = owners
    }

    get tracks() {
        return this.#tracks
    }

    has(track: CachedTrack) {
        return this.#tracks.some((o) => o.path === track.path)
    }

    add(tracks: CachedTrack) {
        if (!this.has(tracks)) {
            this.#tracks.push(tracks)
        }
    }

    delete(tracks: CachedTrack) {
        this.#tracks = this.#tracks.filter((o) => o.path !== tracks.path)
    }

    difference(other: TrackSet) {
        return new TrackSet(
            this.#tracks.filter(
                (track) => !other.#tracks.some((o) => track.path === o.path)
            )
        )
    }

    union(other: TrackSet) {
        let newSet = new TrackSet(this.#tracks)
        for (const track of other) {
            newSet.add(track)
        }

        return newSet
    }

    get size() {
        return this.#tracks.length
    }

    *[Symbol.iterator]() {
        for (const track of this.#tracks) {
            yield track
        }
    }

    // TrackSets should essentially behave like arrays
    toJSON() {
        return this.#tracks
    }
}
