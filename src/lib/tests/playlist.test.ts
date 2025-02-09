import { describe, expect, it } from 'vitest'
import type { Track } from '../types'
import { Playlist, SortMethod, SortOrder } from '$lib/state/playlist.svelte'

const mockTrack1: Track = {
    uuid: '1',
    title: 'Track 1',
    album: 'Album 1',
    artist: 'Artist 1',
    duration: 24,
    path: 'path1/filename1.ogg',
    filename: 'filename1.ogg'
}
const mockTrack2: Track = {
    uuid: '2',
    title: 'Track 2',
    album: 'Album 2',
    artist: 'Artist 2',
    duration: 83,
    path: 'path2/filename2.flac',
    filename: 'filename2.flac'
}
const mockTrack3: Track = {
    uuid: '3',
    title: 'Track 3',
    album: 'Album 3',
    artist: 'Artist 3',
    duration: 100,
    path: 'path3/filename3.mp3',
    filename: 'filename3.mp3'
}
const mockTrack4: Track = {
    uuid: '4',
    title: 'Track 4',
    album: 'Album 4',
    artist: 'Artist 4',
    duration: 11,
    path: 'path4/filename4.wav',
    filename: 'filename4.wav'
}

describe('Playlist class', () => {
    it('constructor initializes with provided arrays and custom getters only return tracks', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = [mockTrack2]
        const priority: Track[] = [mockTrack3]
        const playlist = new Playlist(queue, previous, priority)

        expect(playlist.queue).toEqual([mockTrack1])
        expect(playlist.previous).toEqual([mockTrack2])
        expect(playlist.priority).toEqual([mockTrack3])

        // Verify internal state initialization
        const state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(0)
        expect(state.queueStartIndex).toBe(0)
    })

    it('next plays priority track first', () => {
        const queue: Track[] = [mockTrack1, mockTrack3]
        const previous: Track[] = []
        const priority: Track[] = [mockTrack2]
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.next()

        // 1 should be moved to previous
        // 2 should be at the front
        expect(playlist.queue).toEqual([mockTrack2, mockTrack3])
        expect(playlist.previous).toEqual([mockTrack1])
        expect(playlist.priority).toEqual([])
        expect(result).toEqual({
            justEnded: mockTrack1,
            nextTrack: mockTrack2
        })
    })

    it('handles compaction when removal threshold is reached', () => {
        const playlist = new Playlist([], [], [])
        // Set threshold for testing
        Playlist.setCompactThreshold(3)

        // Add tracks and remove them to trigger compaction
        for (let i = 0; i < 4; i++) {
            playlist.enqueue({ ...mockTrack1, uuid: `test${i}` })
        }

        // Remove tracks one by one
        for (let i = 0; i < 3; i++) {
            playlist.next()
        }

        // Verify internal state before final removal
        let state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(3)
        expect(state.queueLength).toBe(4)

        // This should trigger compaction
        playlist.next()

        // Verify compaction occurred
        state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(0)
        expect(state.queueLength).toBe(0)
        expect(playlist.queue).toEqual([])
    })

    it('next plays queue track if no priority tracks', () => {
        const queue: Track[] = [mockTrack1, mockTrack2, mockTrack3]
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.next()

        expect(playlist.queue).toEqual([mockTrack2, mockTrack3])
        expect(playlist.previous).toEqual([mockTrack1])
        expect(playlist.priority).toEqual([])
        expect(result).toEqual({
            justEnded: mockTrack1,
            nextTrack: mockTrack2
        })

        // Verify internal state
        const state = playlist.getInternalState()
        expect(state.removedQueueIndices.has(0)).toBe(true)
    })

    it('next does nothing if both queue and priority are empty', () => {
        const queue: Track[] = []
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.next()

        expect(playlist.queue).toEqual([])
        expect(playlist.previous).toEqual([])
        expect(playlist.priority).toEqual([])
        expect(result).toBeUndefined()
    })

    it('clear should remove everything including tracking state', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = [mockTrack2]
        const priority: Track[] = [mockTrack3]
        const playlist = new Playlist(queue, previous, priority)

        // Add some removed indices
        playlist.next()

        playlist.clear()

        expect(playlist.queue).toEqual([])
        expect(playlist.previous).toEqual([])
        expect(playlist.priority).toEqual([])

        // Verify tracking state is cleared
        const state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(0)
        expect(state.queueStartIndex).toBe(0)
    })

    it('isEmpty returns true when both queue and priority are empty', () => {
        const queue: Track[] = []
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        expect(playlist.isEmpty()).toBe(true)
    })

    it('isEmpty returns false when queue or priority has tracks', () => {
        const playlist = new Playlist([mockTrack1], [], [])
        expect(playlist.isEmpty()).toBe(false)

        const playlist2 = new Playlist([], [], [mockTrack2])
        expect(playlist2.isEmpty()).toBe(false)

        const playlist3 = new Playlist([mockTrack1], [], [mockTrack2])
        expect(playlist3.isEmpty()).toBe(false)
    })

    it('current returns the first non-removed track in queue', () => {
        const queue: Track[] = [mockTrack1, mockTrack2]
        const playlist = new Playlist(queue, [], [])

        expect(playlist.current()).toEqual(mockTrack1)

        // After removing first track
        playlist.next()
        expect(playlist.current()).toEqual(mockTrack2)
    })

    it('current returns undefined when all tracks are removed', () => {
        const playlist = new Playlist([mockTrack1], [], [])
        playlist.next() // Remove the track
        expect(playlist.current()).toBeUndefined()
    })

    it('last returns the last played track', () => {
        const playlist = new Playlist([], [mockTrack1], [])
        expect(playlist.last()).toEqual(mockTrack1)
    })

    it('last returns undefined if there is no last played track', () => {
        const playlist = new Playlist([], [], [])
        expect(playlist.last()).toBeUndefined()
    })

    it('queued returns all non-removed queued and priority tracks', () => {
        const queue: Track[] = [mockTrack1, mockTrack2]
        const priority: Track[] = [mockTrack3]
        const previous: Track[] = [mockTrack4]
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.queued()
        expect(result.queued).toEqual([mockTrack1, mockTrack2])
        expect(result.priority).toEqual([mockTrack3])

        // After removing some tracks
        playlist.next()
        const updatedResult = playlist.queued()
        expect(updatedResult.queued).toEqual([mockTrack3, mockTrack2])
        expect(updatedResult.priority).toEqual([])
    })

    it('tracks returns all non-removed tracks', () => {
        const queue: Track[] = [mockTrack2, mockTrack3]
        const previous: Track[] = [mockTrack1]
        const priority: Track[] = [mockTrack4]
        const playlist = new Playlist(queue, previous, priority)

        const inPlaylist = playlist.tracks()
        expect(inPlaylist.tracks).toEqual([mockTrack1, mockTrack2, mockTrack3])
        // Technically the equality above is faulty as it's comparing CachedTracks
        // with Tracks (which have a UUID) but it's still a correct result

        // After removing a track (nothing should change)
        playlist.next()
        const updatedPlaylist = playlist.tracks()
        expect(updatedPlaylist.tracks).toEqual([
            mockTrack1,
            mockTrack2,
            mockTrack3
        ])
    })

    it('enqueue adds track to queue', () => {
        const playlist = new Playlist([], [], [])

        playlist.enqueue(mockTrack1)
        expect(playlist.queue).toEqual([mockTrack1])

        playlist.enqueue(mockTrack2)
        expect(playlist.queue).toEqual([mockTrack1, mockTrack2])

        playlist.enqueue(mockTrack3)
        playlist.enqueue(mockTrack4)
        expect(playlist.queue).toEqual([
            mockTrack1,
            mockTrack2,
            mockTrack3,
            mockTrack4
        ])
    })

    it('enqueueFront adds track to front', () => {
        const playlist = new Playlist([mockTrack1, mockTrack2], [], [])

        playlist.enqueueFront(mockTrack3)
        expect(playlist.queue).toEqual([mockTrack3, mockTrack1, mockTrack2])

        playlist.enqueueFront(mockTrack4)
        expect(playlist.queue).toEqual([
            mockTrack4,
            mockTrack3,
            mockTrack1,
            mockTrack2
        ])
    })

    it('enqueuePriority adds track to priority queue', () => {
        const playlist = new Playlist([], [], [])

        playlist.enqueuePriority(mockTrack1)
        expect(playlist.priority).toEqual([mockTrack1])

        // Priority tracks should be AFTER other priority tracks
        playlist.enqueuePriority(mockTrack2)
        expect(playlist.priority).toEqual([mockTrack1, mockTrack2])
    })

    it('overwriteCurrent replaces first non-removed track', () => {
        const queue: Track[] = [mockTrack1, mockTrack2]
        const playlist = new Playlist(queue, [], [])

        // Remove first track
        playlist.next()

        // Overwrite current (should overwrite mockTrack2)
        playlist.overwriteCurrent(mockTrack3)

        expect(playlist.queue).toEqual([mockTrack3])
        expect(playlist.previous).toEqual([mockTrack1, mockTrack2])
    })

    it('sortByUuids handles removed tracks correctly', () => {
        const playlist = new Playlist(
            [mockTrack3, mockTrack1, mockTrack2],
            [],
            []
        )

        // Remove first track
        playlist.next()

        const uuids = ['1', '2']
        playlist.sortAsUuids(uuids)
        expect(playlist.queue).toEqual([mockTrack1, mockTrack2])

        // Verify tracking state is reset
        const state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(0)
        expect(state.queueStartIndex).toBe(0)
    })

    it('alphabetical sort works', () => {
        const trackA = { ...mockTrack1, title: 'A' }
        const trackB = { ...mockTrack2, title: 'B' }
        const trackC = { ...mockTrack3, title: 'C' }
        const trackD = { ...mockTrack4, title: 'D' }

        const playlist = new Playlist([trackD, trackB], [trackC, trackA], [])

        playlist.sortByMethod(SortMethod.Alphabetical, SortOrder.Ascending)
        // The current track (D) should remain at the front
        expect(playlist.queue).toEqual([trackD, trackA, trackB, trackC])
        expect(playlist.previous).toEqual([])

        // Verify tracking state is reset
        const state = playlist.getInternalState()
        expect(state.removedQueueIndices.size).toBe(0)
        expect(state.queueStartIndex).toBe(0)
    })
})
