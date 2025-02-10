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

    getByIdentifier(id: TrackIdentifiers, opts?: { fuzzy?: boolean }) {
        for (const track of this.#tracks) {
            const log = (type: string) => {
                console.debug('Matched with:', type)
                console.debug('- ID:', id)
                console.debug('- TRACK:', $state.snapshot(track))
            }
            // Good, exact matches are marked as reliable
            let reliable = true

            // 1. Check by title and album, if they exist
            if (
                id.title &&
                track.title &&
                id.album &&
                track.album &&
                id.title === track.title &&
                id.album === track.album
            ) {
                log('Title+Album')
                return { track, reliable }
            }

            // 2. Check by title and artist, if they exist
            if (
                id.title &&
                track.title &&
                id.artist &&
                track.artist &&
                id.title === track.title &&
                id.artist === track.artist
            ) {
                log('Title+Artist')
                return { track, reliable }
            }

            reliable = false
            // 3. Check by title only, if it exists
            if (id.title && track.title && id.title === track.title) {
                log('Title Exact')
                return { track, reliable }
            }

            // 4. Check by title only, if it exists, with fuzzy matching
            if (id.title && track.title && opts?.fuzzy) {
                const fuseTitle = new Fuse([track], {
                    keys: ['title'],
                    threshold: 0.05
                })
                const searchResult = fuseTitle.search(id.title)
                if (searchResult.length > 0) {
                    log('Title Fuzzy')
                    return { track, reliable }
                }
            }

            // 5. Check by filename
            if (id.filename === track.filename) {
                log('Filename Exact')
                return { track, reliable }
            }

            // 6. Check by filename, with fuzzy matching
            if (opts?.fuzzy) {
                const fuseFilename = new Fuse([track], {
                    keys: ['filename'],
                    threshold: 0.05
                })
                const searchResult = fuseFilename.search(id.filename)
                if (searchResult.length > 0) {
                    log('Filename Fuzzy')
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
