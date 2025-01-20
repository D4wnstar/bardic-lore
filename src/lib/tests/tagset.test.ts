import { TrackSet, TagSet } from '$lib/state.svelte'
import type { CachedTrack, Tag } from '$lib/types'
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
        expect(tagSet.getByValue('Rock')?.owners.tracks).toEqual([owner1])
    })

    it('should add a new tag', () => {
        const tagSet = new TagSet([rockTag])
        tagSet.add(jazzTag)
        expect(tagSet.has(jazzTag)).toBe(true)
    })

    it('should add owners to an existing tag', () => {
        const tagSet = new TagSet([rockTag])
        const newOwner: CachedTrack = {
            path: '/path3/filename3.flac',
            title: 'Song 3',
            artist: 'Artist 3',
            album: 'Album 3',
            filename: 'filename3.flac'
        }
        tagSet.addOwners({ value: 'Rock', owners: new TrackSet([newOwner]) })
        expect(tagSet.getByTrack(newOwner).has(rockTag)).toBe(true)
    })

    it('should delete owners from an existing tag', () => {
        const tagSet = new TagSet([popTag])
        tagSet.deleteOwners({ value: 'Pop', owners: new TrackSet([owner1]) })
        expect(tagSet.getByTrack(owner1).has(popTag)).toBe(false)
        expect(tagSet.getByTrack(owner2).has(popTag)).toBe(true)
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
        expect(foundTags.has(rockTag)).toBe(true)
        expect(foundTags.has(popTag)).toBe(true)
        expect(foundTags.has(jazzTag)).toBe(false)
    })

    it('should find tags by owner with title and album', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleAlbumOwner1: CachedTrack = {
            path: '/different/path/filename1.ogg',
            title: 'Song 1',
            album: 'Album 1',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(titleAlbumOwner1)
        expect(foundTags.has(rockTag)).toBe(true)
        expect(foundTags.has(popTag)).toBe(true)
        expect(foundTags.has(jazzTag)).toBe(false)
    })

    it('should find tags by owner with title and artist', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleArtistOwner1: CachedTrack = {
            path: '/different/path/filename1.ogg',
            title: 'Song 1',
            artist: 'Artist 1',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(titleArtistOwner1)
        expect(foundTags.has(rockTag)).toBe(true)
        expect(foundTags.has(popTag)).toBe(true)
        expect(foundTags.has(jazzTag)).toBe(false)
    })

    it('should find tags by owner with title only', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const titleOwner1: CachedTrack = {
            path: '/different/path/filename1.ogg',
            title: 'Song 1',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(titleOwner1)
        expect(foundTags.has(rockTag)).toBe(true)
        expect(foundTags.has(popTag)).toBe(true)
        expect(foundTags.has(jazzTag)).toBe(false)
    })

    it('should find tags by owner with filename', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const filenameOwner1: CachedTrack = {
            path: '/different/path/filename1.ogg',
            filename: 'filename1.ogg'
        }
        const foundTags = tagSet.getByTrack(filenameOwner1)
        expect(foundTags.has(rockTag)).toBe(true)
        expect(foundTags.has(popTag)).toBe(true)
        expect(foundTags.has(jazzTag)).toBe(false)
    })

    it('should serialize to JSON correctly', () => {
        const tagSet = new TagSet([rockTag, jazzTag, popTag])
        const serialized = JSON.stringify(tagSet)
        expect(serialized).toBe(JSON.stringify([rockTag, jazzTag, popTag]))
    })
})

describe('TrackSet', () => {
    let owner1: CachedTrack
    let owner2: CachedTrack

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
    })

    it('should initialize with given owners', () => {
        const trackSet = new TrackSet([owner1, owner2])
        expect(trackSet.has(owner1)).toBe(true)
        expect(trackSet.has(owner2)).toBe(true)
    })

    it('should check if an owner exists', () => {
        const trackSet = new TrackSet([owner1])
        expect(trackSet.has(owner1)).toBe(true)
        expect(trackSet.has(owner2)).toBe(false)
    })

    it('should add a new owner', () => {
        const trackSet = new TrackSet([owner1])
        trackSet.add(owner2)
        expect(trackSet.has(owner2)).toBe(true)
    })

    it('should delete an owner', () => {
        const trackSet = new TrackSet([owner1, owner2])
        trackSet.delete(owner1)
        expect(trackSet.has(owner1)).toBe(false)
        expect(trackSet.has(owner2)).toBe(true)
    })

    it('should not add duplicate owners', () => {
        const trackSet = new TrackSet([owner1])
        trackSet.add(owner1)
        expect(trackSet.has(owner1)).toBe(true)
    })

    it('should serialize to JSON correctly', () => {
        const trackSet = new TrackSet([owner1, owner2])
        const serialized = JSON.stringify(trackSet)
        expect(serialized).toBe(JSON.stringify([owner1, owner2]))
    })

    it('should iterate in the standard way', () => {
        const trackSet = new TrackSet([owner1, owner2])
        for (const owner of trackSet) {
            expect(owner).toBeDefined()
        }
    })

    it('should handle empty sets correctly', () => {
        const trackSet = new TrackSet([])
        expect(trackSet.has(owner1)).toBe(false)
        expect(trackSet.tracks).toEqual([])
    })
})
