import { LoopState, Player } from '$lib/state/player.svelte'
import { describe, beforeEach, it, expect } from 'vitest'

describe('Player tests', () => {
    let player: Player

    beforeEach(() => {
        player = new Player({
            playing: false,
            position: 0,
            volume: 50,
            loopState: LoopState.None,
            mute: false,
            shuffle: false
        })
    })

    it('should initialize with correct values', () => {
        expect(player.playing).toBe(false)
        expect(player.position).toBe(0)
        expect(player.volume).toBe(50)
        expect(player.loopState).toBe(LoopState.None)
        expect(player.mute).toBe(false)
        expect(player.mute).toBe(false)
    })

    it('should start playing', () => {
        player.start()
        expect(player.playing).toBe(true)
    })

    it('should stop playing', () => {
        player.start()
        player.stop()
        expect(player.playing).toBe(false)
    })

    it('should reset position and stop playing', () => {
        player.start()
        player.reset()
        expect(player.playing).toBe(false)
        expect(player.position).toBe(0)
    })

    it('should increment position while playing', () => {
        player.start()
        setTimeout(() => {
            expect(player.position).toBeGreaterThan(0)
        }, 2000)
    })
})
