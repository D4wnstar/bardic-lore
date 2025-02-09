import type { CachedTrack, Tag } from '$lib/types'
import Fuse from 'fuse.js'

/**
 * A """`Set`""" of `Tag` objects. JavaScript does not offer deep equality
 * checking for objects in a set and tags need to be unique, so this is a
 * "fake" set that's actually an array where existence is checked manually.
 */
export class TagSet {
    #tags: Tag[] = $state([])
    // #byPath: SvelteMap<string, TagSet> = new SvelteMap()

    constructor(tags: Tag[]) {
        this.#tags = tags
    }

    get tags() {
        return this.#tags
    }

    get size() {
        return this.#tags.length
    }

    has(tag: Tag) {
        return this.#tags.some((t) => t.value === tag.value)
    }

    add(tag: Tag) {
        if (!this.has(tag)) {
            // If the tag doesn't exist it, add it whole
            this.#tags.push(tag)
        } else {
            // If it does exist, this behaves like addOwners
            this.addOwners(tag)
        }
    }

    addOwners(tag: Tag) {
        const existingTag = this.#tags.find((t) => t.value === tag.value)
        if (existingTag) {
            // If it exists, add the owners if they don't exist
            for (const owner of tag.owners) {
                existingTag.owners.add(owner)
            }
        }
    }

    delete(tag: Tag) {
        this.#tags = this.#tags.filter((t) => t.value !== tag.value)
    }

    deleteOwners(tag: Tag) {
        const existingTag = this.#tags.find((t) => t.value === tag.value)
        if (existingTag) {
            // If it exists, delete the owners if they exist
            for (const owner of tag.owners) {
                existingTag.owners.delete(owner)
            }
        }
    }

    difference(other: TagSet) {
        return new TagSet(
            this.#tags.filter(
                (t) => !other.#tags.some((o) => o.value === t.value)
            )
        )
    }

    union(other: TagSet) {
        let newSet = new TagSet(this.#tags)
        for (const tag of other) {
            newSet.add(tag)
        }

        return newSet
    }

    filter(filterFn: (t: Tag) => boolean) {
        const newSet = new TagSet([])
        for (const tag of this.#tags) {
            if (filterFn(tag)) {
                newSet.add(tag)
            }
        }

        return newSet
    }

    get(tagText: string) {
        return this.#tags.find((t) => t.value === tagText)
    }

    /**
     * Find all the tags owned by the given track.
     * @param track The track to search by
     * @param opts Options
     * @returns A `TagSet` with all the found tags
     */
    getByTrack(
        track: CachedTrack,
        opts?: { force?: boolean; fuzzy?: boolean }
    ) {
        // Check if there are cached tags under the owner's path
        // const cachedTags = this.#byPath.get(owner.path)
        // if (cachedTags && !opts?.force) {
        //     return cachedTags
        // }

        let foundTags: TagSet = new TagSet([])
        for (const tag of this.#tags) {
            // For every tag, check if the given owner owns it
            for (const tagOwner of tag.owners) {
                // 1. Check by path
                if (tagOwner.path === track.path) {
                    foundTags.add(tag)
                    break
                }

                // 2. Check by title and album, if they exist
                if (
                    track.title &&
                    track.album &&
                    tagOwner.title === track.title &&
                    tagOwner.album === track.album
                ) {
                    foundTags.add(tag)
                    break
                }

                // 3. Check by title and artist, if they exist
                if (
                    track.title &&
                    track.artist &&
                    tagOwner.title === track.title &&
                    tagOwner.artist === track.artist
                ) {
                    foundTags.add(tag)
                    break
                }

                // 4. Check by title only, if it exists
                if (track.title && tagOwner.title === track.title) {
                    foundTags.add(tag)
                    break
                }

                // 5. Check by title only, if it exists, with fuzzy matching
                if (track.title && opts?.fuzzy) {
                    const fuseTitle = new Fuse([tagOwner], {
                        keys: ['title'],
                        threshold: 0.3
                    })
                    const searchResult = fuseTitle.search(track.title)
                    if (searchResult.length > 0) {
                        foundTags.add(tag)
                    }
                }

                // 6. Check by filename
                if (tagOwner.filename === track.filename) {
                    foundTags.add(tag)
                    break
                }

                // 7. Check by filename, with fuzzy matching
                if (opts?.fuzzy) {
                    const fuseFilename = new Fuse([tagOwner], {
                        keys: ['filename'],
                        threshold: 0.1
                    })
                    const searchResult = fuseFilename.search(track.filename)
                    if (searchResult.length > 0) {
                        foundTags.add(tag)
                    }
                }
            }
        }

        // Update the cache with the found tracks
        // this.#byPath.set(owner.path, foundTags)

        return foundTags
    }

    /**
     * @returns An array of tags sorted alphabetically in ascending order
     */
    sorted() {
        return this.#tags.toSorted((a, b) => a.value.localeCompare(b.value))
    }

    *[Symbol.iterator]() {
        for (const tag of this.#tags) {
            yield tag
        }
    }

    // TagSets should essentially behave like arrays
    toJSON() {
        return this.#tags
    }
}
