import type { CachedTrack } from '$lib/types'
import Fuse from 'fuse.js'
import type { TrackIdentifiers } from './taggroupset.svelte'

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

    get size() {
        return this.#tracks.length
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

    getByIdentifier(
        id: TrackIdentifiers,
        opts?: { force?: boolean; fuzzy?: boolean }
    ) {
        for (const track of this.#tracks) {
            // Good, exact matches are marked as reliable
            let reliable = true

            // 1. Check by title and album, if they exist
            if (
                track.title &&
                track.album &&
                id.title === track.title &&
                id.album === track.album
            ) {
                return { track, reliable }
            }

            // 2. Check by title and artist, if they exist
            if (
                track.title &&
                track.artist &&
                id.title === track.title &&
                id.artist === track.artist
            ) {
                return { track, reliable }
            }

            reliable = false
            // 3. Check by title only, if it exists
            if (track.title && id.title === track.title) {
                return { track, reliable }
            }

            // 4. Check by title only, if it exists, with fuzzy matching
            if (track.title && opts?.fuzzy) {
                const fuseTitle = new Fuse([id], {
                    keys: ['title'],
                    threshold: 0.3
                })
                const searchResult = fuseTitle.search(track.title)
                if (searchResult.length > 0) {
                    return { track, reliable }
                }
            }

            // 5. Check by filename
            if (id.filename === track.filename) {
                return { track, reliable }
            }

            // 6. Check by filename, with fuzzy matching
            if (opts?.fuzzy) {
                const fuseFilename = new Fuse([id], {
                    keys: ['filename'],
                    threshold: 0.1
                })
                const searchResult = fuseFilename.search(track.filename)
                if (searchResult.length > 0) {
                    return { track, reliable }
                }
            }
        }
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
