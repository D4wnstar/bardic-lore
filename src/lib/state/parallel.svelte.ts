import type { Track } from '$lib/types'
import type { Player } from './player.svelte'

export type ParallelState = {
    track: Track
    player: Player
}

export class Parallel {
    states: ParallelState[] = $state([])

    constructor(states: ParallelState[]) {
        this.states = states
    }

    get tracks() {
        return this.states.map((s) => s.track)
    }

    get players() {
        return this.states.map((s) => s.player)
    }

    /**
     * Find the parallel `Player` associated with the track of the given path.
     * @param path The filepath to search by
     * @returns A `Player`, if any was found
     */
    getPlayerByUuid(uuid: string): Player | undefined {
        const maybePlayer = this.states.find(({ track }) => track.uuid === uuid)

        if (!maybePlayer) {
            console.warn(`Failed to find track with UUID ${uuid}`)
            return
        }

        return maybePlayer.player
    }

    clear() {
        for (const state of this.states) {
            state.player.stop()
        }
        this.states = []
    }
}
