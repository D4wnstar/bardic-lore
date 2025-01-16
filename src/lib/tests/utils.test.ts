import { rgbToHex, permuteTracks } from '../utils/utils'
import type { Track } from '../types'
import { describe, it, expect } from 'vitest'

const mockTrack1: Track = {
    title: 'Track1',
    album: 'Album1',
    artist: 'Artist1',
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
        const tracks: Track[] = [mockTrack1, mockTrack2, mockTrack3]
        const result = permuteTracks(mockTrack2, tracks)
        expect(result).toEqual([mockTrack2, mockTrack3, mockTrack1])
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
        const tracks: Track[] = [mockTrack1, mockTrack2, mockTrack3]
        const result = permuteTracks(anotherTrack, tracks)
        expect(result).toBeUndefined()
    })
})
