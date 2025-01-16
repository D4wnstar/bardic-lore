import { LoopState, Parallel, Player } from '$lib/state.svelte'
import type { Track } from '$lib/types'
import { beforeEach, describe, expect, it } from 'vitest'

describe('Parallel', () => {
    const mockPlayer1 = new Player({
        playing: false,
        position: 0,
        loopState: LoopState.None,
        mute: false,
        shuffle: false,
        volume: 0.5
    })
    const mockPlayer2 = new Player({
        playing: true,
        position: 14,
        loopState: LoopState.LoopTrack,
        mute: false,
        shuffle: true,
        volume: 0.42
    })

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
        album: 'Album2',
        artist: 'Artist2',
        duration: 123,
        extension: 'ogg',
        path: '/path/to/file2.ogg',
        uuid: '456-def'
    }

    let parallel: Parallel

    beforeEach(() => {
        parallel = new Parallel([
            { track: mockTrack1, player: mockPlayer1 },
            { track: mockTrack2, player: mockPlayer2 }
        ])
    })

    it('should construct correctly', () => {
        expect(parallel.states).toEqual([
            { track: mockTrack1, player: mockPlayer1 },
            { track: mockTrack2, player: mockPlayer2 }
        ])
    })

    it('getPlayerByUuid should return the correct Player', () => {
        const player = parallel.getPlayerByUuid('123-abc')
        expect(player).toEqual(mockPlayer1)
    })

    it('getPlayerByUuid should return undefined if no player is found', () => {
        const player = parallel.getPlayerByUuid('8190428014')
        expect(player).toBeUndefined()
    })

    it('clear should reset everything', () => {
        parallel.clear()
        expect(parallel.states).toEqual([])
    })
})
