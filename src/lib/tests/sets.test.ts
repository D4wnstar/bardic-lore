import { TrackSet, TagSet, TagGroupSet } from '$lib/state.svelte'
import type { CachedTrack, Tag, TagGroup } from '$lib/types'
import { describe, it, expect, beforeEach } from 'vitest'

describe('TagSet', () => {
    let owner1: CachedTrack
    let owner2: CachedTrack
    let rockTag: Tag
    let jazzTag: Tag
    let popTag: Tag

    beforeEach(() => {
        owner1 = {
            path: '/path1/filename1.ogg',
            title: 'Song 1',
            artist: 'Artist 1',
            album: 'Album 1',
            filename: 'filename1.ogg'
        }
        owner2 = {
            path: '/path2/filename2.mp3',
            title: 'Song 2',
            artist: 'Artist 2',
            album: 'Album 2',
            filename: 'filename2.mp3'
        }

        rockTag = { value: 'Rock', owners: new TrackSet([owner1]) }
        jazzTag = { value: 'Jazz', owners: new TrackSet([]) }
        popTag = {
            value: 'Pop',
            owners: new TrackSet([owner1, owner2])
        }
    })

    // Owner1 has Rock and Pop
    // Owner2 has Pop

    it('should initialize with given tags', () => {
        const tagSet = new TagSet([rockTag, jazzTag])
        expect(tagSet.has(rockTag)).toBe(true)
        expect(tagSet.has(jazzTag)).toBe(true)
    })

    it('should check if a tag exists', () => {
        const tagSet = new TagSet([rockTag])
        expect(tagSet.has(rockTag)).toBe(true)
        expect(tagSet.has(jazzTag)).toBe(false)
        expect(tagSet.get('Rock')?.owners.tracks).toEqual([owner1])
    })

    it('should add a new tag', () => {
        const tagSet = new TagSet([rockTag])
        tagSet.add(jazzTag)
        expect(tagSet.has(jazzTag)).toBe(true)
    })

    it('should add owners to an existing tag', () => {
        const tagSet = new TagSet([rockTag, popTag, jazzTag])
        const newOwner: CachedTrack = {
            path: '/path3/filename3.flac',
            title: 'Song 3',
            artist: 'Artist 3',
            album: 'Album 3',
            filename: 'filename3.flac'
        }
        tagSet.addOwners({ value: 'Rock', owners: new TrackSet([newOwner]) })
        expect(tagSet.get('Rock')?.owners.tracks).toEqual([owner1, newOwner])
        expect(tagSet.get('Pop')?.owners.tracks).toEqual([owner1, owner2])
        expect(tagSet.get('Jazz')?.owners.tracks).toEqual([])
    })

    it('should delete owners from an existing tag', () => {
        const tagSet = new TagSet([rockTag, popTag, jazzTag])
        tagSet.deleteOwners({ value: 'Pop', owners: new TrackSet([owner1]) })
        expect(tagSet.get('Rock')?.owners.tracks).toEqual([owner1])
        expect(tagSet.get('Pop')?.owners.tracks).toEqual([owner2])
        expect(tagSet.get('Jazz')?.owners.tracks).toEqual([])
    })

    it('should not add duplicate tags', () => {
        const tagSet = new TagSet([rockTag])
        tagSet.add(rockTag)
        expect(tagSet.has(rockTag)).toBe(true)
    })

    it('should delete a tag', () => {
        const tagSet = new TagSet([rockTag, jazzTag])
        tagSet.delete(rockTag)
        expect(tagSet.has(rockTag)).toBe(false)
        expect(tagSet.has(jazzTag)).toBe(true)
    })

    it('should perform union with another array of tags', () => {
        const tagSet = new TagSet([rockTag])
        const newSet = tagSet.union(new TagSet([jazzTag, popTag]))
        expect(newSet.tags).toEqual([rockTag, jazzTag, popTag])
    })

    it('should perform difference with another array of tags', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const newSet = tagSet.difference(new TagSet([jazzTag]))
        expect(newSet.tags).toEqual([rockTag, popTag])
    })

    it('should iterate in the standard way', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        for (const tag of tagSet) {
            expect(tag).toBeDefined()
        }
    })

    it('should find tags by owner with path', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const pathOwner1: CachedTrack = {
            path: '/path1/filename1.ogg',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(pathOwner1)
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should find tags by owner with title and album', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleAlbumOwner1: CachedTrack = {
            path: '/different/path/wrong_name.ogg',
            title: 'Song 1',
            album: 'Album 1',
            filename: 'wrong_name.ogg'
        }
        const foundTags = tagSet.getByTrack(titleAlbumOwner1)
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should find tags by owner with title and artist', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleArtistOwner1: CachedTrack = {
            path: '/different/path/wrong_name.ogg',
            title: 'Song 1',
            artist: 'Artist 1',
            filename: 'wrong_name.ogg'
        }
        const foundTags = tagSet.getByTrack(titleArtistOwner1)
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should find tags by owner with title only (exact)', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleOwner1: CachedTrack = {
            path: '/different/path/wrong_name.ogg',
            title: 'Song 1',
            filename: 'wrong_name.ogg'
        }
        const foundTags = tagSet.getByTrack(titleOwner1)
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should find tags by owner with title only (fuzzy)', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleOwner1: CachedTrack = {
            path: '/different/path/wrong_name.ogg',
            title: 'SongED 1',
            filename: 'wrong_name.ogg'
        }
        const foundTags = tagSet.getByTrack(titleOwner1, { fuzzy: true })
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should find tags by owner with filename', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const filenameOwner1: CachedTrack = {
            path: '/different/path/filename1.ogg',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(filenameOwner1)
        expect(foundTags.tags).toEqual([rockTag, popTag])
    })

    it('should serialize to JSON correctly', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const serialized = JSON.stringify(tagSet)
        expect(serialized).toBe(JSON.stringify([rockTag, jazzTag, popTag]))
    })
})

describe('TrackSet', () => {
    let track1: CachedTrack
    let track2: CachedTrack

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
    })

    it('should initialize with given tracks', () => {
        const trackSet = new TrackSet([track1, track2])
        expect(trackSet.has(track1)).toBe(true)
        expect(trackSet.has(track2)).toBe(true)
    })

    it('should check if an track exists', () => {
        const trackSet = new TrackSet([track1])
        expect(trackSet.has(track1)).toBe(true)
        expect(trackSet.has(track2)).toBe(false)
    })

    it('should add a new track', () => {
        const trackSet = new TrackSet([track1])
        trackSet.add(track2)
        expect(trackSet.has(track2)).toBe(true)
    })

    it('should delete an track', () => {
        const trackSet = new TrackSet([track1, track2])
        trackSet.delete(track1)
        expect(trackSet.has(track1)).toBe(false)
        expect(trackSet.has(track2)).toBe(true)
    })

    it('should not add duplicate tracks', () => {
        const trackSet = new TrackSet([track1])
        trackSet.add(track1)
        expect(trackSet.has(track1)).toBe(true)
    })

    it('should serialize to JSON correctly', () => {
        const trackSet = new TrackSet([track1, track2])
        const serialized = JSON.stringify(trackSet)
        expect(serialized).toBe(JSON.stringify([track1, track2]))
    })

    it('should iterate in the standard way', () => {
        const trackSet = new TrackSet([track1, track2])
        for (const track of trackSet) {
            expect(track).toBeDefined()
        }
    })

    it('should handle empty sets correctly', () => {
        const trackSet = new TrackSet([])
        expect(trackSet.has(track1)).toBe(false)
        expect(trackSet.tracks).toEqual([])
    })
})

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

        rockTag = { value: 'Rock', owners: new TrackSet([track1]) }
        jazzTag = { value: 'Jazz', owners: new TrackSet([track2]) }
        popTag = { value: 'Pop', owners: new TrackSet([]) }

        group1 = { name: 'Group 1', tagSet: new TagSet([rockTag, jazzTag]) }
        group2 = { name: 'Group 2', tagSet: new TagSet([popTag]) }
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
        const newTag: Tag = { value: 'New Tag', owners: new TrackSet([]) }
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
            owners: new TrackSet([newOwner])
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
            owners: new TrackSet([track1])
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
