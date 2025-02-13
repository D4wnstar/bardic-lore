import { type TagGroup, type Tag, type CachedTrack } from '$lib/types'
import { TagSet } from './tagset.svelte'
import { TrackSet } from './trackset.svelte'

/**
 * A list of properties to identify a track.
 */
export type TrackIdentifiers = {
    title?: string
    album?: string
    artist?: string
    filename: string
}

/**
 * A hacky """`Set`""" to keep `TagGroups`s and their contained tags unique.
 */
export class TagGroupSet {
    #groups: TagGroup[] = $state([])

    /**
     * The name of the default `TagGroup` that's should be guaranteed
     * to exist.
     */
    static DEFAULT_GROUP = 'Uncategorized'
    /**
     * The name of the built-in album group.
     */
    static ALBUM_GROUP = 'Albums'
    /**
     * The name of the built-in artist group.
     */
    static ARTIST_GROUP = 'Artists'

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

    /**
     * Add a new group into the set. Does nothing if the group already exists.
     * @param group The TagGroup to add
     */
    add(group: TagGroup) {
        if (!this.has(group)) {
            this.#groups.push(group)
        }
    }

    /**
     * Delete a group from the set. Does nothing if the group doesn't exists.
     * @param group The TagGroup to delete
     */
    delete(groupName: string) {
        this.#groups = this.#groups.filter((g) => g.name !== groupName)
    }

    difference(other: TagGroupSet) {
        const difference = new TagGroupSet([])
        for (const group of other) {
            if (this.has(group)) {
                difference.add(group)
                for (const tag of group.tagSet) {
                    group.tagSet.delete(tag)
                }
            } else {
                difference.add(group)
            }
        }
        return difference
    }

    differenceTags(otherTags: TagSet) {
        const difference = new TagGroupSet([])
        this.#groups.map((g) => difference.add(g))
        for (const tag of otherTags) {
            difference.deleteTag(tag)
        }
        return difference
    }

    /**
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
     * @param tagName The tag to search for
     * @returns A `Tag`, if any was found
     */
    getTag(tagName: string) {
        let foundTag: Tag | undefined
        for (const group of this.#groups) {
            let maybeTag = group.tagSet.get(tagName)
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

    getByTrack(path: string) {
        let totalTagSet = new TagSet([])
        for (const group of this.#groups) {
            const partialTagSet = group.tagSet.getByTrack(path)
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
     * @param tagName The tag to use. The owners of this tag will be merged with existing ones
     * @returns `true` if the group was found, `false` if it wasn't
     */
    addTagOwners(groupName: string, tagName: string, tracks: CachedTrack[]) {
        const groupToAddTo = this.get(groupName)
        if (groupToAddTo) {
            groupToAddTo.tagSet.addOwners(tagName, tracks)
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
            this.deleteTag(maybeTag, maybeTag.group as string)
            this.addTag(groupName, maybeTag)
            return true
        } else {
            return false
        }
    }

    /**
     * Delete a tag from a group.
     * @param tag The tag to delete
     * @param groupName The group to delete from. Leave empty for any group
     */
    deleteTag(tag: Tag, groupName?: string) {
        if (groupName) {
            const groupToDeleteFrom = this.get(groupName)
            if (groupToDeleteFrom) {
                groupToDeleteFrom.tagSet.delete(tag)
            }
        } else {
            for (const group of this.#groups) {
                group.tagSet.delete(tag)
            }
        }
    }

    /**
     * Delete owners from a tag in a group.
     * @param groupName The group to search the tag in
     * @param tag The tag to use. The owners of this tag will be removed from existing ones
     */
    deleteTagOwners(groupName: string, tagName: string, tracks: CachedTrack[]) {
        const groupToDeleteFrom = this.get(groupName)
        if (groupToDeleteFrom) {
            groupToDeleteFrom.tagSet.deleteOwners(tagName, tracks)
        }
    }

    sorted() {
        const defaultGroup = this.#groups.find(
            (g) => g.name === TagGroupSet.DEFAULT_GROUP
        )
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

    /**
     * @returns A JSON string with all exported tags. Ignores unmodifiable
     * groups like albums and artists.
     */
    export() {
        const toExport = this.#groups.filter((g) => g.modifiable)

        for (const group of toExport) {
            for (const tag of group.tagSet) {
                //@ts-expect-error We overwrite each TrackSet with an array of TrackIdentifiers
                // in order to strip unnecessary and possibly private information (like filepaths)
                tag.owners = tag.owners.tracks.map<TrackIdentifiers>((t) => {
                    return {
                        title: t.title,
                        album: t.album,
                        artist: t.artist,
                        filename: t.filename
                    }
                })
            }
        }

        // The exported schema is the same as a TagGroupSet but with TrackSets
        // changed to TrackIdentifiers[]
        return JSON.stringify(toExport, null, 2)
    }

    /**
     * Imports tags and groups into Bardic Lore from serialized JSON.
     * @param jsonString A string containing serialized JSON, as given by the `export` function
     * @param tracks The tracks to match to.
     */
    import(jsonString: string, tracks: TrackSet) {
        const importedTagGroups: TagGroup[] = JSON.parse(jsonString)
        // Note that JavaScript can't serialize into a functioning class,
        // and also export() converts CachedTracks into TrackIdentifiers, so
        // TagSets are actually just Tag[] and TrackSets are TrackIdentifiers[]
        // TypeScript doesn't know so make sure you don't accidentally call a
        // class function or property that doesn't exist

        // Each group should be added or merged with an existing one
        // Each tag should be added or merged with an existing one
        // The TrackIdentifiers in each tag should be matched with a track
        // that is currently loaded in the app. If an exact match is found, convert
        // the identifier into that track. If there is a partial match (i.e. from fuzzy
        // search), warn the user and give them a selection of most likely candidates.
        // If there is no match, warn the user and ask them to match the track manually
        // or ignore it.

        // The actual user invertention logic should be somewhere in the GUI, so we
        // return a data structure detailing what needs to be handled.

        for (const group of importedTagGroups) {
            // Import each group into the app, starting with no tags
            this.add({ ...group, tagSet: new TagSet([]) })

            //@ts-expect-error JavaScript can't parse classes
            const tags = group.tagSet as Tag[]
            for (const partialTag of tags) {
                // Import each tag into the new group, starting with no owners
                this.addTag(group.name, {
                    ...partialTag,
                    owners: new TrackSet([])
                })

                //@ts-expect-error JavaScript can't parse classes
                const owners = partialTag.owners as TrackIdentifiers[]
                for (const ownerIds of owners) {
                    // Try to match each owner identifier with an existing track
                    const result = tracks.getByIdentifier(ownerIds, {
                        fuzzy: false
                    })

                    if (result) {
                        // If one is found, add the owner to the new tag
                        if (!result.reliable) {
                            // TODO: Implement the mechanism for user intervention on unreliable matches
                            console.warn(`Unreliable match on ${ownerIds}`)
                        }
                        this.addTagOwners(group.name, partialTag.value, [
                            result.track
                        ])
                    }
                }
            }
        }
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
