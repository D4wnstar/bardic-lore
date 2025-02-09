import type { TagSet } from './state/tagset.svelte'
import type { TrackSet } from './state/trackset.svelte'

/**
 * Information about an audio source and its state.
 */
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
    title?: string
    album?: string
    artist?: string
    duration?: number
    path: string
    filename: string
    coverHash?: string
}

/**
 * Data about a track, like `CachedTrack` except it also includes a UUID
 * to identify the track in the Discord bot. Meant to be used every time
 * a track is actually played or waiting to be played, such as in the queue.
 */
export type Track = {
    uuid: string
    title?: string
    album?: string
    artist?: string
    duration?: number
    path: string
    filename: string
    coverHash?: string
}

/**
 * A track with a mask flag.
 */
export type MaskedTrack = {
    track: CachedTrack
    visible: boolean
}

/**
 * Information about a Discord guild. Contains a list of voice channels.
 */
export type GuildSlug = {
    id: number
    name: string
    voiceChannels: VoiceChannelSlug[]
    offline: boolean
}

/**
 * Information about a Discord voice channel.
 */
export type VoiceChannelSlug = {
    id: number
    name: string
    active: boolean
}

/**
 * A tag for organization and filtering. The `value` is the actual text of the
 * tag, whereas the "owners" are all the tracks that have this tag on them.
 * `group` is the name of the group the `TagGroup` the tag is in.
 */
export type Tag = {
    value: string
    owners: TrackSet
    group: string
}

/**
 * A group of related tags identified by a name.
 */
export type TagGroup = {
    name: string
    tagSet: TagSet
    builtin: boolean
    modifiable: boolean
}

/**
 * The name of the default `TagGroup` that's should be guaranteed
 * to exist.
 */
export const DEFAULT_GROUP = 'Uncategorized'
/**
 * The name of the built-in album group.
 */
export const ALBUM_GROUP = 'Albums'
/**
 * The name of the built-in artist group.
 */
export const ARTIST_GROUP = 'Artists'
