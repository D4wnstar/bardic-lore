import {
    type TagGroup,
    type Tag,
    type CachedTrack,
    DEFAULT_GROUP
} from '$lib/types'
import { TagSet } from './tagset.svelte'

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
