export type AudioSource = {
    path: string
    recursive: boolean
    active: boolean
}

/**
 * Data about a track as found in the file metadata. Meant to be used when
 * a track only needs to be displayed in the UI, such as in buttons. When the
 * track is actually sent to the Discord client, it should be added to the queue
 * as a `Track` instead, which also contains a UUID to identify it.
 */
export type CachedTrack = {
    title: string
    album: string | undefined
    artist: string | undefined
    duration: number | undefined
    path: string
    extension: string
}

/**
 * Data about a track, like `CachedTrack` except it also includes a UUID
 * to identify the track in the Discord bot. Meant to be used every time
 * a track is actually played or waiting to be played, such as in the queue.
 */
export type Track = {
    uuid: string
    title: string
    album: string | undefined
    artist: string | undefined
    duration: number | undefined
    path: string
    extension: string
}

// serde_json can't know what's inside of a JSON value
// so we send numbers as string to be deserialized into a
// HashMap<String, String>
export type GuildSlug = {
    id: number
    name: string
    voice_channels: VoiceChannelSlug[]
    offline: boolean
}

export type VoiceChannelSlug = {
    id: number
    name: string
    active: boolean
}
