import { TagGroupSet } from '$lib/state/taggroupset.svelte'
import { TagSet } from '$lib/state/tagset.svelte'
import { TrackSet } from '$lib/state/trackset.svelte'
import { type CachedTrack, type Tag, type TagGroup } from '$lib/types'
import { describe, it, expect, beforeEach } from 'vitest'

describe('TagGroupSet', () => {
    let track1: CachedTrack
    let track2: CachedTrack
    let rockTag: Tag
    let jazzTag: Tag
    let popTag: Tag
    let group1: TagGroup
    let group2: TagGroup

    beforeEach(() => {
        track1 = {
            path: '/path1/filename1.ogg',
            title: 'Song 1',
            artist: 'Artist 1',
            album: 'Album 1',
            filename: 'filename1.ogg'
        }
        track2 = {
            path: '/path2/filename2.mp3',
            title: 'Song 2',
            artist: 'Artist 2',
            album: 'Album 2',
            filename: 'filename2.mp3'
        }

        rockTag = {
            value: 'Rock',
            owners: new TrackSet([track1]),
            group: 'Group 1'
        }
        jazzTag = {
            value: 'Jazz',
            owners: new TrackSet([track2]),
            group: 'Group 1'
        }
        popTag = {
            value: 'Pop',
            owners: new TrackSet([]),
            group: 'Group 2'
        }

        group1 = {
            name: 'Group 1',
            tagSet: new TagSet([rockTag, jazzTag]),
            builtin: false,
            modifiable: true
        }
        group2 = {
            name: 'Group 2',
            tagSet: new TagSet([popTag]),
            builtin: false,
            modifiable: true
        }
    })

    it('should initialize with given groups', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        expect(tagGroupSet.has(group1)).toBe(true)
        expect(tagGroupSet.has(group2)).toBe(true)
    })

    it('should check if a group exists', () => {
        const tagGroupSet = new TagGroupSet([group1])
        expect(tagGroupSet.has(group1)).toBe(true)
        expect(tagGroupSet.has(group2)).toBe(false)
    })

    it('should add a new group', () => {
        const tagGroupSet = new TagGroupSet([group1])
        tagGroupSet.add(group2)
        expect(tagGroupSet.has(group2)).toBe(true)
    })

    it('should add a tag to a group', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        const newTag: Tag = {
            value: 'New Tag',
            owners: new TrackSet([]),
            group: TagGroupSet.DEFAULT_GROUP
        }
        tagGroupSet.addTag('Group 1', newTag)

        const addedTag = tagGroupSet.get('Group 1')?.tagSet.get('New Tag')
        expect(addedTag).toBeDefined()
        // Also check that the added tag's group field is coerced to the new group
        expect(addedTag?.group).toEqual('Group 1')
    })

    it('should add owners to a tag in a group', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        const newOwner: CachedTrack = {
            path: '/path3/filename3.flac',
            title: 'Song 3',
            artist: 'Artist 3',
            album: 'Album 3',
            filename: 'filename3.flac'
        }
        tagGroupSet.addTagOwners('Group 1', {
            value: 'Rock',
            owners: new TrackSet([newOwner]),
            group: TagGroupSet.DEFAULT_GROUP
        })
        expect(
            tagGroupSet.get('Group 1')?.tagSet.get('Rock')?.owners.tracks
        ).toEqual([track1, newOwner])
    })

    it('should delete a tag from a group', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        tagGroupSet.deleteTag('Group 1', rockTag)
        expect(tagGroupSet.get('Group 1')?.tagSet.has(rockTag)).toBe(false)
    })

    it('should delete owners from a tag in a group', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        tagGroupSet.deleteTagOwners('Group 1', {
            value: 'Rock',
            owners: new TrackSet([track1]),
            group: TagGroupSet.DEFAULT_GROUP
        })
        expect(
            tagGroupSet.get('Group 1')?.tagSet.get('Rock')?.owners.tracks
        ).toEqual([])
    })

    it('should get a tag by searching across all groups', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        const foundTag = tagGroupSet.getTag('Pop')
        expect(foundTag).toEqual(popTag)
    })

    it('should get tags by track', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        const foundTags = tagGroupSet.getByTrack(track1)
        expect(foundTags.tags).toEqual([rockTag])
    })

    it('should serialize to JSON correctly', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        const serialized = JSON.stringify(tagGroupSet)
        expect(serialized).toBe(JSON.stringify([group1, group2]))
    })

    it('should iterate in the standard way', () => {
        const tagGroupSet = new TagGroupSet([group1, group2])
        for (const group of tagGroupSet) {
            expect(group).toBeDefined()
        }
    })
})
