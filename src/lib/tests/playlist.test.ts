import { expect, test } from 'vitest'
import { Playlist } from '../state.svelte'
import type { Track } from '../types'

const mockTrack1: Track = {
    uuid: '1',
    title: 'Track 1',
    album: undefined,
    artist: undefined,
    duration: undefined,
    path: 'path1',
    extension: 'ext1'
}
const mockTrack2: Track = {
    uuid: '2',
    title: 'Track 2',
    album: undefined,
    artist: undefined,
    duration: undefined,
    path: 'path2',
    extension: 'ext2'
}
const mockTrack3: Track = {
    uuid: '3',
    title: 'Track 3',
    album: undefined,
    artist: undefined,
    duration: undefined,
    path: 'path3',
    extension: 'ext3'
}

test('constructor initializes with provided arrays', () => {
    const queue: Track[] = [mockTrack1]
    const previous: Track[] = [mockTrack2]
    const priority: Track[] = [mockTrack3]
    const playlist = new Playlist(queue, previous, priority)

    expect(playlist.queue).toEqual([{ track: mockTrack1, singleUse: false }])
    expect(playlist.previous).toEqual([{ track: mockTrack2, singleUse: false }])
    expect(playlist.priority).toEqual([{ track: mockTrack3, singleUse: true }])
})

test('next plays priority track first', () => {
    const queue: Track[] = [mockTrack1]
    const previous: Track[] = []
    const priority: Track[] = [mockTrack2]
    const playlist = new Playlist(queue, previous, priority)

    const result = playlist.next()

    expect(playlist.queue).toEqual([{ track: mockTrack2, singleUse: true }])
    expect(playlist.previous).toEqual([{ track: mockTrack1, singleUse: false }])
    expect(playlist.priority).toEqual([])
    expect(result).toEqual({
        justEnded: { track: mockTrack1, singleUse: false },
        nextTrack: { track: mockTrack2, singleUse: true }
    })
})

test('next plays queue track if no priority tracks', () => {
    const queue: Track[] = [mockTrack1]
    const previous: Track[] = []
    const priority: Track[] = []
    const playlist = new Playlist(queue, previous, priority)

    const result = playlist.next()

    expect(playlist.queue).toEqual([])
    expect(playlist.previous).toEqual([{ track: mockTrack1, singleUse: false }])
    expect(playlist.priority).toEqual([])
    expect(result).toEqual({
        justEnded: { track: mockTrack1, singleUse: false },
        nextTrack: undefined
    })
})

test('next does nothing if both queue and priority are empty', () => {
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

test('clear should remove everything', () => {
    const queue: Track[] = [mockTrack1]
    const previous: Track[] = [mockTrack2]
    const priority: Track[] = [mockTrack3]
    const playlist = new Playlist(queue, previous, priority)

    playlist.clear()

    expect(playlist.queue).toEqual([])
    expect(playlist.previous).toEqual([])
    expect(playlist.priority).toEqual([])
})

test('isEmpty returns true when both queue and priority are empty', () => {
    const queue: Track[] = []
    const previous: Track[] = []
    const priority: Track[] = []
    const playlist = new Playlist(queue, previous, priority)

    expect(playlist.isEmpty()).toBe(true)
})

test('isEmpty returns false when queue or priority has tracks', () => {
    const queue: Track[] = [mockTrack1]
    const previous: Track[] = []
    const priority: Track[] = []
    const playlist = new Playlist(queue, previous, priority)

    expect(playlist.isEmpty()).toBe(false)

    const playlist2 = new Playlist([], [], [mockTrack2])
    expect(playlist2.isEmpty()).toBe(false)
})

test('current returns the first track in queue', () => {
    const queue: Track[] = [mockTrack1, mockTrack2]
    const playlist = new Playlist(queue, [], [])

    expect(playlist.current()).toEqual(mockTrack1)
})

test('current returns undefined when queue is empty', () => {
    const playlist = new Playlist([], [], [])
    expect(playlist.current()).toBeUndefined()
})

test('queued returns all queued and priority tracks', () => {
    const queue: Track[] = [mockTrack1]
    const priority: Track[] = [mockTrack2]
    const playlist = new Playlist(queue, [], priority)

    const result = playlist.queued()
    expect(result.queued).toEqual([mockTrack1])
    expect(result.priority).toEqual([mockTrack2])
})

test('enqueue adds track to queue', () => {
    const playlist = new Playlist([], [], [])
    playlist.enqueue(mockTrack1)

    expect(playlist.queue).toEqual([{ track: mockTrack1, singleUse: false }])
})

test('enqueuePriority adds track to priority queue', () => {
    const playlist = new Playlist([], [], [])
    playlist.enqueuePriority(mockTrack1)

    expect(playlist.priority).toEqual([{ track: mockTrack1, singleUse: true }])
})

test('overwriteCurrent replaces current track and pushes old to previous', () => {
    const queue: Track[] = [mockTrack1]
    const playlist = new Playlist(queue, [], [])

    playlist.overwriteCurrent(mockTrack2)

    expect(playlist.queue[0]).toEqual({ track: mockTrack2, singleUse: false })
    expect(playlist.previous).toEqual([{ track: mockTrack1, singleUse: false }])
})

test('overwriteCurrent does nothing when queue is empty', () => {
    const playlist = new Playlist([], [], [])
    playlist.overwriteCurrent(mockTrack1)

    expect(playlist.queue).toEqual([])
    expect(playlist.previous).toEqual([])
})
