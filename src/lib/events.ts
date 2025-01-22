// This file contains events emitted by Tauri for communication, primarily
// between the Discord bot and the UI to keep them synchronized.

import type { CachedTrack, Track } from './types'

// All events should be written in kebab-case.
// This file is a direct copy of src/events.rs file. If you need to add or change an event,
// please change the Rust file too. The syntax between the two is the same except
// 'pub' -> 'export' and ': &str' is removed. Doc comment format also changes.

/* FROM UI TO BOT */
/**
 * This event tells the Bot to enter a voice channel.
 * The payload must include the guild ID and the channel ID.
 */
export const JOIN_VOICE_CHANNEL = 'join-voice-channel'
/**
 * This event tells the Bot to leave the voice channel
 * it is currently in. The payload must include the guild ID.
 */
export const LEAVE_VOICE_CHANNEL = 'leave-voice-channel'
/**
 * This event tells the bot to add a track to the queue.
 * The payload must include the guild ID and a Track object.
 */
export const QUEUE_TRACK = 'queue-track'
/**
 * This event tells the bot to fill the queue with all the given tracks
 * after clearing the current queue.
 */
export const CREATE_PLAYLIST = 'create-playlist'
/**
 * This event tells the bot to play a track on top of existing ones.
 * It will not be added to the queue. The payload must include the guild ID
 * and a Track object.
 */
export const PLAY_PARALLEL = 'play-parallel'
/**
 * This event tells the bot to update one or more of the tracks. Whether the
 * track is in the queue or in parallel and what action to take depend on the
 * contents of the payload.
 */
export const UPDATE_TRACKS = 'update-tracks'
/**
 * This event tells the bot to resume playback of its queue.
 * If playback is not paused or the queue is empty, it does nothing.
 * The payload must include the guild ID.
 */
/**
 * This event tells the bot to mute or unmute, inverting the state.
 * The payload must include the guild ID.
 */
export const MUTE_UNMUTE = 'mute-unmute'

/* FROM BOT TO UI */
/**
 * This event indicates that the client succesfully connected to Discord.
 */
export const CLIENT_CONNECTED = 'client-connected'
/**
 * This event indicates that there was an error in a bot command. It is
 * generic and applies for any error related to the serenity client.
 * The payload should contain more information.
 */
export const BOT_ERROR = 'bot-error'
/**
 * This event indicates that the bot received information on a new guild
 * and the Tauri store was update to match.
 * It is fired whenever the bot receives a GUILD_CREATE event.
 */
export const UPDATED_GUILDS = 'updated-guilds'
/**
 * This event indicates that the bot left its voice channel and either joined
 * a new one or it went offline.
 */
export const LEFT_VOICE_CHANNEL = 'left-voice-channel'
/**
 * This event instructs the frontend to update the playerState $state rune
 * using the information passed in the payload. The payload must be a `serde_json`
 * `Value`, probably made with the `json!` macro. The frontend will update the
 * fields in the $state based on which keys match. See src/lib/stores.svelte.ts
 * for the data structure. Make sure the types are correct.
 */
export const UPDATE_PLAYER = 'update-player'
/**
 * This event instructs the frontend to add the track given in the payload to
 * either the main queue or the parallel tracks, depending on what the payload says.
 * By track here we mean a `crate::files::Track`, not a serenity `Track`.
 */
export const ADD_TRACK = 'add-track'
/**
 * This event notifies that a playlist was created and the queue was filled. Must send
 * the list of all tracks that were added.
 */
export const PLAYLIST_CREATED = 'playlist-created'
/**
 * This event notifies that the queue has been emptied.
 */
export const QUEUE_EMPTIED = 'queue-emptied'
/**
 * This event notifies that the queue has been sorted or shuffled.
 */
export const QUEUE_SORTED = 'queue-sorted'

/* TRACKEVENT RELAYS */
/**
 * This event notifies the frontend that a track just finished. Essentially a relay
 * of songbird's `TrackEvent::End`.
 */
export const TRACK_ENDED = 'track-ended'
/**
 * This event notifies the frontend that a track just looped. Essentially a relay
 * of songbird's `TrackEvent::Loop`.
 */
export const TRACK_LOOPED = 'track-looped'
/**
 * This event notifies the frontend that a track just became playable. Essentially a relay
 * of songbird's `TrackEvent::Playable`.
 */
export const TRACK_PLAYABLE = 'track-playable'
/**
 * This event notifies the frontend that a track just resumed playing. Essentially a relay
 * of songbird's `TrackEvent::Play`.
 */
export const TRACK_PLAYED = 'track-played'
/**
 * This event notifies the frontend that a track just paused. Essentially a relay
 * of songbird's `TrackEvent::Pause`.
 */
export const TRACK_PAUSED = 'track-paused'

/* PAYLOADS FROM TAURI */
/**
 * This enum describes the method used to add a track to the queue.
 */
export enum QueueMethod {
    Normal,
    Priority,
    Backskip,
    OverwriteCurrent
}

export enum TrackAction {
    Resume,
    Pause,
    Skip,
    Stop,
    Loop,
    Seek,
    ChangeVolume,
    Shuffle,
    Sort
}

export type AddTrackPayload = {
    track: Track
    parallel: boolean
    queueMethod: QueueMethod
    looping: boolean
}

export type TrackEventPayload = {
    isParallel: boolean
    uuid: string
}

/* PAYLOADS TO TAURI */
// To use these properly, use the `satisfies` TypeScript keyword
export type CreatePlaylistPayload = {
    guildId: number
    tracksData: CachedTrack[]
    volume: number
    loopFirst: boolean
    shuffle: boolean
}

export type QueueTrackPayload = {
    guildId: number
    trackData: CachedTrack
    looping: boolean
    queueMethod: QueueMethod
    volume: number
}

export type PlayParallelPayload = {
    guildId: number
    trackData: CachedTrack
    looping: boolean
    volume: number
}

export type TrackActionPayload = {
    guildId: number
    action: TrackAction
    parallel: boolean
    /**
     * Mandatory if `parallel` is true.
     */
    uuid?: string
    /**
     * Mandatory for a seek action.
     */
    position?: number
    /**
     * Mandatory for a change volume action.
     */
    volume?: number
    /**
     * Mandatory for a sort action.
     */
    sortUuids?: string[]
}

export type PlaylistCreatedPayload = {
    tracks: Track[]
}

export type QueueShuffledPayload = {
    uuids: string[]
}
