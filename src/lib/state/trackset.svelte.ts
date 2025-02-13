import type { CachedTrack } from '$lib/types'
import Fuse from 'fuse.js'
import type { TrackIdentifiers } from './taggroupset.svelte'
import type { TagSet } from './tagset.svelte'

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

    visible() {
        return this.#tracks.filter((t) => t.visible)
    }

    /**
     * Set the visibility of tracks in the set. The `filter` function will be ran on each track
     * and its output determines the visibility of that track.
     * @param filter The filter function
     */
    setVisibility(filter: (t: CachedTrack) => boolean) {
        this.#tracks = this.#tracks.map((track) => {
            return {
                ...track,
                visible: filter(track)
            }
        })
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
        return new TrackSet(this.#tracks.filter((track) => !other.has(track)))
    }

    union(other: TrackSet) {
        let newSet = new TrackSet(this.#tracks)
        for (const track of other) {
            newSet.add(track)
        }

        return newSet
    }

    clear() {
        this.#tracks = []
    }

    filter(tags: TagSet, mode: 'any' | 'all', searchTerm?: string) {
        // First, run string similarity search with Fuse
        const beforeSearch = Date.now()
        let searchedTracks: CachedTrack[]
        if (searchTerm && searchTerm.length > 0) {
            const fuse = new Fuse(this.#tracks, {
                keys: ['title', 'filename'],
                threshold: 0.3
            })
            searchedTracks = fuse.search(searchTerm).map((res) => res.item)
        } else {
            searchedTracks = this.#tracks
        }
        const afterSearch = Date.now()

        // Hide all tracks that did not match the search
        this.difference(new TrackSet(searchedTracks)).#tracks.map(
            (t) => (t.visible = false)
        )

        // Then remove all tracks that lack the given tags
        const beforeTags = Date.now()
        for (const track of searchedTracks) {
            let foundTag = true
            if (tags.size > 0) {
                if (mode === 'all') {
                    foundTag = tags.tags.every((tag) =>
                        track.tags.has(tag.value)
                    )
                } else if (mode === 'any') {
                    foundTag = tags.tags.some((tag) =>
                        track.tags.has(tag.value)
                    )
                }
            }

            track.visible = foundTag
        }
        const afterTags = Date.now()

        console.log(
            `Title search took ${afterSearch - beforeSearch} ms, tag search took ${afterTags - beforeTags} ms`
        )
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

    /**
     * Find a track by its filename. Useful as a faster alternative to `getByIdentifier`
     * if you already know the filename exists.
     * @param filename The filename to search for
     */
    getByFilename(filename: string) {
        return this.#tracks.find((t) => {
            return t.filename === filename
        })
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
