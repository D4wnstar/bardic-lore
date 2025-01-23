import Fuse from 'fuse.js'
import {
    DEFAULT_GROUP,
    type CachedTrack,
    type Tag,
    type TagGroup,
    type Track
} from './types'

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
        return this.#previous.at(-1)?.track
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
     * Returns a list of all of the tracks that make up the playlist.
     * @returns An array of `Track`s
     */
    tracks() {
        const previous = this.#previous.map((t) => t.track)
        const queued = this.#queue.map((t) => t.track)
        return [...previous, ...queued]
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
        } else {
            nextTrack = this.#queue.at(0)
        }

        return { justEnded: justEnded.track, nextTrack: nextTrack?.track }
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
     * Pops the previous queue.
     * @returns The popped track, if any
     */
    popPrevious() {
        return this.#previous.pop()?.track
    }

    /**
     * Check if the queue is empty by checking both the normal and priority queues.
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
     * Sorts the queue so that the tracks follow the same order as the one given by the `uuids` array.
     * @param uuids The array of UUIDs to sort by
     * @param opts Whether to skip the first (and current) track
     * If this is true, the given UUIDs should NOT contain the UUID of the first track
     * @returns The sorted array of tracks
     */
    sortByUuids(uuids: string[], opts = { skipFirst: false }) {
        let tracks = opts.skipFirst ? this.#queue.slice(1) : this.#queue
        const sortedTracks = uuids
            .map((uuid) => {
                const track = tracks.find((t) => t.track.uuid === uuid)
                if (!track) {
                    console.warn(`Failed to find track with UUID ${uuid}`)
                    return null
                }
                return track
            })
            .filter((track) => track !== null)

        if (opts.skipFirst) {
            this.#queue = [this.#queue[0], ...sortedTracks]
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
        if (this.tracks().length < 2) return

        let sortedTracks = [...this.#previous, ...this.#queue]
        this.#previous = []
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

        if (this.#queue[0]) {
            // Make sure to rotate the array so that the current track is at the start
            sortedTracks = this.permuteTracks(
                this.#queue[0],
                sortedTracks
            ) as TrackInternal[]
        }

        this.#queue = sortedTracks
    }

    private permuteTracks(startTrack: TrackInternal, tracks: TrackInternal[]) {
        let maybeIndex = tracks.findIndex((t) => t === startTrack)
        if (maybeIndex === -1) return

        let firstBlock = tracks.slice(maybeIndex)
        let secondBlock = tracks.slice(0, maybeIndex)
        return [...firstBlock, ...secondBlock]
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
    shuffle = $state(false)
    mute = $state(false)
    private timerId: number | undefined = $state()

    constructor(opts: {
        playing: boolean
        position: number
        volume: number
        loopState: LoopState
        shuffle: boolean
        mute: boolean
    }) {
        this.#playing = opts.playing
        this.position = opts.position
        this.volume = opts.volume
        this.loopState = opts.loopState
        this.shuffle = this.shuffle
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

export type ParallelState = {
    track: Track
    player: Player
}

export class Parallel {
    states: ParallelState[] = $state([])

    constructor(states: ParallelState[]) {
        this.states = states
    }

    get tracks() {
        return this.states.map((s) => s.track)
    }

    get players() {
        return this.states.map((s) => s.player)
    }

    /**
     * Find the parallel `Player` associated with the track of the given path.
     * @param path The filepath to search by
     * @returns A `Player`, if any was found
     */
    getPlayerByUuid(uuid: string): Player | undefined {
        const maybePlayer = this.states.find(({ track }) => track.uuid === uuid)

        if (!maybePlayer) {
            console.warn(`Failed to find track with UUID ${uuid}`)
            return
        }

        return maybePlayer.player
    }

    clear() {
        for (const state of this.states) {
            state.player.stop()
        }
        this.states = []
    }
}

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

/**
 * A hacky """`Set`""" to keep `TagGroups`s and their contained tags unique.
 */
export class TagGroupSet {
    #groups: TagGroup[] = $state([])

    constructor(groups: TagGroup[]) {
        this.#groups = groups
    }

    get groups() {
        return this.#groups
    }

    get size() {
        return this.#groups.length
    }

    has(group: TagGroup) {
        return this.#groups.some((g) => g.name === group.name)
    }

    add(group: TagGroup) {
        if (!this.has(group)) {
            this.#groups.push(group)
        }
    }

    delete(groupName: string) {
        this.#groups = this.#groups.filter((g) => g.name !== groupName)
    }

    /**
     *
     * @param oldName The current name of the group to rename
     * @param newName The new name to give it
     * @returns `true` if renaming was successful, `false` if no group called `oldName`
     * was found or if there already is a group with the new name
     */
    rename(oldName: string, newName: string) {
        if (oldName === newName) {
            return true
        }

        const toChange = this.get(oldName)
        const maybeExisting = this.get(newName)
        if (maybeExisting) {
            console.warn(
                `Tried to rename group ${oldName} to an existing name (${newName})`
            )
            return false
        }

        if (toChange) {
            toChange.name = newName
            return true
        } else {
            return false
        }
    }

    get(groupName: string) {
        return this.#groups.find((g) => g.name === groupName)
    }

    /**
     * Get a tag by searching across all groups.
     * @param tagText The tag to search for
     * @returns A `Tag`, if any was found
     */
    getTag(tagText: string) {
        let foundTag: Tag | undefined
        for (const group of this.#groups) {
            let maybeTag = group.tagSet.get(tagText)
            if (maybeTag) {
                // Just in case, we coerce the tag's group to be the one we got it from
                foundTag = {
                    ...maybeTag,
                    group: group.name
                }
                break
            }
        }

        return foundTag
    }

    getByTrack(
        track: CachedTrack,
        opts?: { force?: boolean; fuzzy?: boolean }
    ) {
        let totalTagSet = new TagSet([])
        for (const group of this.#groups) {
            const partialTagSet = group.tagSet.getByTrack(track, opts)
            // Just in case, we coerce the tag's group to be the one we got it from
            partialTagSet.tags.forEach((tag) => (tag.group = group.name))
            totalTagSet = totalTagSet.union(partialTagSet)
        }

        return totalTagSet
    }

    /**
     * Add a tag to a group. If the tag already exists in a different group,
     * it'll be moved from that group to the given one.
     * @param groupName The group to add to
     * @param tag The tag to add
     * @returns `true` if the group was found, `false` if it wasn't
     */
    addTag(groupName: string, tag: Tag) {
        const existingTag = this.getTag(tag.value)
        const groupToAddTo = this.get(groupName)

        if (groupToAddTo) {
            const newTag: Tag = {
                ...tag,
                group: groupToAddTo.name
            }

            if (existingTag) {
                this.moveTag(groupName, newTag)
            } else {
                groupToAddTo.tagSet.add(newTag)
            }
            return true
        } else {
            return false
        }
    }

    /**
     * Add owners to a tag in a group.
     * @param groupName The group to search the tag in
     * @param tag The tag to use. The owners of this tag will be merged with existing ones
     * @returns `true` if the group was found, `false` if it wasn't
     */
    addTagOwners(groupName: string, tag: Tag) {
        const groupToAddTo = this.get(groupName)
        if (groupToAddTo) {
            groupToAddTo.tagSet.addOwners({
                ...tag,
                group: groupToAddTo.name
            })
            return true
        } else {
            return false
        }
    }

    /**
     * Move the given tag from its current group to the one with the given name.
     * The tag is searched across all groups, so its `group` field doesn't need to
     * be correct.
     * @param groupName The group to the move the tag to
     * @param tag The tag to move
     * @returns `true` if a tag got moved, `false` if it didn't
     */
    moveTag(groupName: string, tag: Tag) {
        const maybeTag = this.getTag(tag.value)
        if (maybeTag && this.get(groupName)) {
            // getTag guarantees that the group field is a string
            this.deleteTag(maybeTag.group as string, maybeTag)
            this.addTag(groupName, maybeTag)
            return true
        } else {
            return false
        }
    }

    /**
     * Delete a tag from a group.
     * @param groupName The group to delete from
     * @param tag The tag to delete
     * @returns `true` if the group was found, `false` if it wasn't
     */
    deleteTag(groupName: string, tag: Tag) {
        const groupToDeleteFrom = this.get(groupName)
        if (groupToDeleteFrom) {
            groupToDeleteFrom.tagSet.delete(tag)
            return true
        } else {
            return false
        }
    }

    /**
     * Delete owners from a tag in a group.
     * @param groupName The group to search the tag in
     * @param tag The tag to use. The owners of this tag will be removed from existing ones
     * @returns `true` if the group was found, `false` if it wasn't
     */
    deleteTagOwners(groupName: string, tag: Tag) {
        const groupToDeleteFrom = this.get(groupName)
        if (groupToDeleteFrom) {
            groupToDeleteFrom.tagSet.deleteOwners(tag)
            return true
        } else {
            return false
        }
    }

    sorted() {
        const defaultGroup = this.#groups.find((g) => g.name === DEFAULT_GROUP)
        const customGroups = this.#groups.filter((g) => !g.builtin)
        const unmodifiableGroups = this.#groups.filter((g) => !g.modifiable)

        return [
            defaultGroup as TagGroup,
            ...customGroups.toSorted((a, b) => a.name.localeCompare(b.name)),
            ...unmodifiableGroups.toSorted((a, b) =>
                a.name.localeCompare(b.name)
            )
        ]
    }

    *[Symbol.iterator]() {
        for (const group of this.#groups) {
            yield group
        }
    }

    // TagGroups should essentially behave like arrays
    toJSON() {
        return this.#groups
    }
}
