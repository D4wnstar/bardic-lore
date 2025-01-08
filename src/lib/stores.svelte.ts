// This module contains constants to access the Tauri stores that Bardic Lore
// uses to keep things synchronized between the front- and backend and also for
// persistent storage.

import type { Track } from './types'

// All keys should be written in kebab-case.
// This file is a direct copy of src/stores.rs file. If you need to add or change a store,
// please change the Rust file too. The syntax between the two is the same except
// 'pub' -> 'export' and ': &str' is removed. Doc comment format also changes.
// Unlike the Rust version, this file also includes global $state runes.

/* SETTINGS */
/**
 * The settings store contains all user settings, such as audio sources
 * and the Discord bot token.
 */
export const SETTINGS_FILENAME = 'settings.json'
export const AUDIO_SOURCES_SETTING = 'audio-sources'
export const BOT_TOKEN_SETTING = 'bot-token'

/* TRACKS */
/**
 * The tracks store is a cache for the tracks found in the audio sources.
 * Tracks should be written to this store once when the sources change and
 * then all access to tracks should be from here. This avoids re-reading
 * possibly hundreds of tracks with attached metadata everytime the app
 * starts.
 */
export const TRACKS_FILENAME = 'tracks.json'
export const TRACKS_SETTING = 'tracks'

/* DISCORD */
/**
 * The Discord store contains data that is shared between the frontend and
 * the backend. Strictly speaking, it does not need to be persistent, as it is
 * regenerated every time the bot connects, but it works well as a debugging aid.
 */
export const DISCORD_FILENAME = 'discord.json'
export const GUILDS_SETTING = 'guilds'

/* SVELTE STATE */
export type MaybeTrack = {
    track: Track | undefined
}

export type PlayerState = {
    playing: boolean
    position: number
    volume: number
    looping: boolean
    offline: boolean
    mute: boolean
    trackQueue: Track[]
    recentlyPlayed: Track[]
    parallelTracks: Track[]
}

export const globalGuild = $state({
    id: 0
})
export const toOverwrite: MaybeTrack = $state({ track: undefined })
export const playerState: PlayerState = $state({
    playing: false,
    position: 0,
    volume: 1.0,
    looping: false,
    offline: true,
    mute: false,
    trackQueue: [],
    recentlyPlayed: [],
    parallelTracks: []
})
