export enum LoopState {
    None,
    LoopPlaylist,
    LoopTrack
}

export class Player {
    #playing = $state(false)
    position = $state(0)
    volume = $state(0.5)
    loopState = $state(LoopState.None)
    shuffle = $state(false)
    mute = $state(false)
    private timerId: number | undefined = $state()

    constructor(opts: {
        playing: boolean
        position: number
        volume: number
        loopState: LoopState
        shuffle: boolean
        mute: boolean
    }) {
        this.#playing = opts.playing
        this.position = opts.position
        this.volume = opts.volume
        this.loopState = opts.loopState
        this.shuffle = this.shuffle
        this.mute = opts.mute
        this.timerId = undefined
    }

    private addTimer() {
        if (!this.timerId) {
            this.timerId = setInterval(() => {
                this.position += 1
            }, 1000)
        }
    }

    private removeTimer() {
        if (this.timerId) {
            clearInterval(this.timerId)
            this.timerId = undefined
        }
    }

    start() {
        this.#playing = true
        this.addTimer()
    }

    stop() {
        this.#playing = false
        this.removeTimer()
    }

    reset() {
        this.#playing = false
        this.removeTimer()
        this.position = 0
    }

    get playing() {
        return this.#playing
    }
}
