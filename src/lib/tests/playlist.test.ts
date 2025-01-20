import { describe, expect, it } from 'vitest'
import { Playlist, SortMethod, SortOrder } from '../state.svelte'
import type { Track } from '../types'

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
    })

    it('next plays priority track first', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = []
        const priority: Track[] = [mockTrack2]
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.next()

        expect(playlist.queue).toEqual([mockTrack2])
        expect(playlist.previous).toEqual([mockTrack1])
        expect(playlist.priority).toEqual([])
        expect(result).toEqual({
            justEnded: { track: mockTrack1, singleUse: false },
            nextTrack: { track: mockTrack2, singleUse: true }
        })
    })

    it('next plays queue track if no priority tracks', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        const result = playlist.next()

        expect(playlist.queue).toEqual([])
        expect(playlist.previous).toEqual([mockTrack1])
        expect(playlist.priority).toEqual([])
        expect(result).toEqual({
            justEnded: { track: mockTrack1, singleUse: false },
            nextTrack: undefined
        })
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

    it('clear should remove everything', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = [mockTrack2]
        const priority: Track[] = [mockTrack3]
        const playlist = new Playlist(queue, previous, priority)

        playlist.clear()

        expect(playlist.queue).toEqual([])
        expect(playlist.previous).toEqual([])
        expect(playlist.priority).toEqual([])
    })

    it('isEmpty returns true when both queue and priority are empty', () => {
        const queue: Track[] = []
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        expect(playlist.isEmpty()).toBe(true)
    })

    it('isEmpty returns false when queue or priority has tracks', () => {
        const queue: Track[] = [mockTrack1]
        const previous: Track[] = []
        const priority: Track[] = []
        const playlist = new Playlist(queue, previous, priority)

        expect(playlist.isEmpty()).toBe(false)

        const playlist2 = new Playlist([], [], [mockTrack2])
        expect(playlist2.isEmpty()).toBe(false)
    })

    it('current returns the first track in queue', () => {
        const queue: Track[] = [mockTrack1, mockTrack2]
        const playlist = new Playlist(queue, [], [])

        expect(playlist.current()).toEqual(mockTrack1)
    })

    it('current returns undefined when queue is empty', () => {
        const playlist = new Playlist([], [], [])
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

    it('queued returns all queued and priority tracks', () => {
        const queue: Track[] = [mockTrack1]
        const priority: Track[] = [mockTrack2]
        const playlist = new Playlist(queue, [], priority)

        const result = playlist.queued()
        expect(result.queued).toEqual([mockTrack1])
        expect(result.priority).toEqual([mockTrack2])
    })

    it('tracks returns all tracks, both previous and queued', () => {
        const playlist = new Playlist(
            [mockTrack2, mockTrack3],
            [mockTrack1],
            [mockTrack2]
        )

        const inPlaylist = playlist.tracks()
        expect(inPlaylist).toEqual([mockTrack1, mockTrack2, mockTrack3])
    })

    it('enqueue adds track to queue', () => {
        const playlist = new Playlist([], [], [])
        playlist.enqueue(mockTrack1)

        expect(playlist.queue).toEqual([mockTrack1])
    })

    it('enqueuePriority adds track to priority queue', () => {
        const playlist = new Playlist([], [], [])
        playlist.enqueuePriority(mockTrack1)

        expect(playlist.priority).toEqual([mockTrack1])
    })

    it('overwriteCurrent replaces current track and pushes old to previous', () => {
        const queue: Track[] = [mockTrack1]
        const playlist = new Playlist(queue, [], [])

        playlist.overwriteCurrent(mockTrack2)

        expect(playlist.queue[0]).toEqual(mockTrack2)
        expect(playlist.previous).toEqual([mockTrack1])
    })

    it('overwriteCurrent does nothing when queue is empty', () => {
        const playlist = new Playlist([], [], [])
        playlist.overwriteCurrent(mockTrack1)

        expect(playlist.queue).toEqual([])
        expect(playlist.previous).toEqual([])
    })

    it('sortByUuids sorts tracks correctly when all UUIDs are present', () => {
        const playlist = new Playlist(
            [mockTrack3, mockTrack1, mockTrack2],
            [],
            []
        )

        const uuids = ['1', '2', '3']
        playlist.sortByUuids(uuids)
        expect(playlist.queue).toEqual([mockTrack1, mockTrack2, mockTrack3])
    })

    it('sortByUuids ignores UUIDs that are not found in the tracks array', () => {
        const playlist = new Playlist([mockTrack3, mockTrack1], [], [])

        const uuids = ['1', '999999999', '3']
        playlist.sortByUuids(uuids)
        expect(playlist.queue).toEqual([mockTrack1, mockTrack3])
    })

    it('sortByUuids removes tracks that do not match any UUID', () => {
        const playlist = new Playlist(
            [mockTrack1, mockTrack2, mockTrack3],
            [],
            []
        )

        const uuids: string[] = []
        playlist.sortByUuids(uuids)
        expect(playlist.queue).toEqual([])
    })

    it('sortByUuids handles a single element array', () => {
        const playlist = new Playlist([mockTrack1], [], [])

        const uuids = ['1']
        playlist.sortByUuids(uuids)
        expect(playlist.queue).toEqual([mockTrack1])
    })

    it('sortByUuids skips the first track if told', () => {
        const playlist = new Playlist(
            [mockTrack2, mockTrack3, mockTrack1],
            [],
            []
        )

        const uuids = ['1', '3']
        playlist.sortByUuids(uuids, { skipFirst: true })
        expect(playlist.queue).toEqual([mockTrack2, mockTrack1, mockTrack3])
    })

    it('sortByMethod correctly sorts alphabetically', () => {
        const trackA = {
            ...mockTrack1,
            title: 'A'
        }
        const trackB = {
            ...mockTrack2,
            title: 'B'
        }
        const trackC = {
            ...mockTrack3,
            title: 'C'
        }
        const trackD = {
            ...mockTrack4,
            title: 'D'
        }
        const previous: Track[] = [trackC, trackA]
        const queue: Track[] = [trackD, trackB]

        const playlist = new Playlist(queue, previous, [])

        playlist.sortByMethod(SortMethod.Alphabetical, SortOrder.Ascending)
        expect(playlist.queue).toEqual([trackD, trackA, trackB, trackC])

        playlist.sortByMethod(SortMethod.Alphabetical, SortOrder.Descending)
        expect(playlist.queue).toEqual([trackD, trackC, trackB, trackA])
    })
})
