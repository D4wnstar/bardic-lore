import { getPlayerByUuid, rgbToHex, permuteTracks } from '../utils/utils'
import { appState } from '../stores.svelte'
import type { Track } from '../types'
import { describe, it, expect } from 'vitest'
import { LoopState, Player } from '$lib/state.svelte'

const mockTrack: Track = {
    title: 'Track',
    album: 'Album',
    artist: 'Artist',
    duration: 123,
    extension: 'ogg',
    path: '/path/to/file.ogg',
    uuid: '123-abc'
}
const mockTrack2: Track = {
    title: 'Track2',
    album: 'Album',
    artist: 'Artist',
    duration: 123,
    extension: 'ogg',
    path: '/path/to/file2.ogg',
    uuid: '456-def'
}
const mockTrack3: Track = {
    title: 'Track3',
    album: 'Album',
    artist: 'Artist',
    duration: 123,
    extension: 'ogg',
    path: '/path/to/file3.ogg',
    uuid: '789-ghi'
}

describe('getPlayerByUuid', () => {
    const mockPlayer = new Player({
        playing: false,
        position: 0,
        mute: false,
        loopState: LoopState.None,
        shuffle: false,
        volume: 0.5
    })

    it('should return the player associated with the given UUID', () => {
        appState.parallelPlayers = [{ player: mockPlayer, track: mockTrack }]

        const result = getPlayerByUuid('123-abc')
        expect(result).toEqual(mockPlayer)
    })

    it('should return undefined if no player is found', () => {
        appState.parallelPlayers = []

        const result = getPlayerByUuid('123-abc')
        expect(result).toBeUndefined()
    })
})

describe('rgbToHex', () => {
    it('should convert RGB string to hexadecimal format', () => {
        const result = rgbToHex('255 128 64')
        expect(result).toBe('#ff8040')
    })

    it('should handle zero values correctly', () => {
        const result = rgbToHex('0 0 0')
        expect(result).toBe('#000000')
    })
})

describe('permuteTracks', () => {
    it('should permute tracks with the start track at the beginning', () => {
        const tracks: Track[] = [mockTrack, mockTrack2, mockTrack3]
        const result = permuteTracks(mockTrack2, tracks)
        expect(result).toEqual([mockTrack2, mockTrack3, mockTrack])
    })

    it('should return undefined if the start track is not found', () => {
        let anotherTrack: Track = {
            title: 'Not in here',
            album: 'Album',
            artist: 'Artist',
            duration: 102,
            extension: 'mp3',
            path: '/somewhere/anywhere.mp3',
            uuid: '999999999'
        }
        const tracks: Track[] = [mockTrack, mockTrack2, mockTrack3]
        const result = permuteTracks(anotherTrack, tracks)
        expect(result).toBeUndefined()
    })
})
