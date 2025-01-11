import { describe, beforeEach, it, expect } from 'vitest'
import { Player } from '../state.svelte'

describe('Player tests', () => {
    let player: Player

    beforeEach(() => {
        player = new Player({
            playing: false,
            position: 0,
            volume: 50,
            looping: false,
            mute: false
        })
    })

    it('should initialize with correct values', () => {
        expect(player.playing).toBe(false)
        expect(player.position).toBe(0)
        expect(player.volume).toBe(50)
        expect(player.looping).toBe(false)
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

    it('should maintain volume setting', () => {
        player.volume = 75
        expect(player.volume).toBe(75)
    })

    it('should toggle looping', () => {
        player.looping = true
        expect(player.looping).toBe(true)
    })

    it('should toggle mute', () => {
        player.mute = true
        expect(player.mute).toBe(true)
    })
})
