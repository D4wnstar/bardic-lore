import { TrackSet } from '$lib/state/trackset.svelte'
import type { CachedTrack } from '$lib/types'
import { describe, beforeEach, it, expect } from 'vitest'

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

    it('should serialize to JSON like an array', () => {
        const trackSet = new TrackSet([track1, track2])
        const serialized = JSON.stringify(trackSet)
        expect(serialized).toBe(JSON.stringify([track1, track2]))
    })

    it('should iterate like an array', () => {
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
