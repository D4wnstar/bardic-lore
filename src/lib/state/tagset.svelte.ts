import type { CachedTrack, Tag } from '$lib/types'

/**
 * A """`Set`""" of `Tag` objects. JavaScript does not offer deep equality
 * checking for objects in a set and tags need to be unique, so this is a
 * "fake" set that's actually an array where existence is checked manually.
 */
export class TagSet {
    #tags: Tag[] = $state([])
    // Cache map to store tags by track path
    #tagsByTrack: Map<string, Tag[]> = new Map()

    constructor(tags: Tag[]) {
        this.#tags = tags
        // Initialize cache for existing tags
        this.#updateCache()
    }

    get tags() {
        return this.#tags
    }

    get size() {
        return this.#tags.length
    }

    has(tagName: string) {
        return this.#tags.some((t) => t.value === tagName)
    }

    #updateCache() {
        this.#tagsByTrack.clear()
        for (const tag of this.#tags) {
            for (const track of tag.owners) {
                const trackTags = this.#tagsByTrack.get(track.path) ?? []
                trackTags.push(tag)
                this.#tagsByTrack.set(track.path, trackTags)
            }
        }
    }

    add(tag: Tag) {
        if (!this.has(tag.value)) {
            // If the tag doesn't exist, add it whole
            this.#tags.push(tag)
            // Update the owners in return and maintain cache
            // for (const track of tag.owners) {
            //     track.tags.add(tag)
            //     const trackTags = this.#tagsByTrack.get(track.path) || []
            //     trackTags.push(tag)
            //     this.#tagsByTrack.set(track.path, trackTags)
            // }
        } else {
            // If it does exist, this behaves like addOwners
            this.addOwners(tag.value, tag.owners.tracks)
        }
    }

    addOwners(tagName: string, tracks: CachedTrack[]) {
        const existingTag = this.#tags.find((t) => t.value === tagName)
        if (existingTag) {
            // If it exists, add the owners
            for (const track of tracks) {
                existingTag.owners.add(track)
                // track.tags.add(existingTag)
                // const trackTags = this.#tagsByTrack.get(track.path) ?? []
                // if (!trackTags.some((t) => t.value === existingTag.value)) {
                //     trackTags.push(existingTag)
                //     this.#tagsByTrack.set(track.path, trackTags)
                // }
            }
        }
    }

    delete(tag: Tag) {
        // Remove tag from all owners' tag sets first
        // const existingTag = this.#tags.find((t) => t.value === tag.value)
        // if (existingTag) {
        //     for (const owner of existingTag.owners) {
        //         owner.tags.delete(existingTag)
        //         const trackTags = this.#tagsByTrack.get(owner.path)
        //         if (trackTags) {
        //             this.#tagsByTrack.set(
        //                 owner.path,
        //                 trackTags.filter((t) => t.value !== tag.value)
        //             )
        //         }
        //     }
        // }
        this.#tags = this.#tags.filter((t) => t.value !== tag.value)
    }

    deleteOwners(tagName: string, tracks: CachedTrack[]) {
        const existingTag = this.#tags.find((t) => t.value === tagName)
        if (existingTag) {
            // If it exists, delete the owners if they exist
            for (const track of tracks) {
                existingTag.owners.delete(track)
                // track.tags.delete(existingTag)
                // const trackTags = this.#tagsByTrack.get(track.path)
                // if (trackTags) {
                //     this.#tagsByTrack.set(
                //         track.path,
                //         trackTags.filter((t) => t.value !== tagName)
                //     )
                // }
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
     * @param path The track path to search by
     * @returns A `TagSet` with all the found tags
     */
    getByTrack(path: string) {
        // Check cache first
        const cachedTags = this.#tagsByTrack.get(path)
        if (cachedTags) {
            return new TagSet(cachedTags)
        }

        // If not in cache (shouldn't happen with proper sync), rebuild cache
        this.#updateCache()
        const rebuiltCachedTags = this.#tagsByTrack.get(path)
        return new TagSet(rebuiltCachedTags ?? [])
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
