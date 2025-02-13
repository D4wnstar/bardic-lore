// This module contains constants to access the Tauri stores that Bardic Lore
// uses to keep things synchronized between the front- and backend and also for
// persistent storage.

import { SvelteMap } from 'svelte/reactivity'
import { type Track } from './types'
import { LoopState, Player } from './state/player.svelte'
import { Parallel } from './state/parallel.svelte'
import { Playlist } from './state/playlist.svelte'
import { TagGroupSet } from './state/taggroupset.svelte'
import { TagSet } from './state/tagset.svelte'
import { TrackSet } from './state/trackset.svelte'

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
export const AUTOCONNECT_SETTING = 'autoconnect'

export const VOLUME_SETTING = 'volume'
export const MUTE_SETTING = 'mute'
export const SHUFFLE_SETTING = 'shuffle'
export const LOOP_SETTING = 'loop'

export const GROUP_ACCORDION_STATES = 'group-accordion-states'

export const DARK_MODE = 'dark-mode'
export const SHOW_COVERS_SETTING = 'show-covers'
export const AUTOHIDE_SIDEBARS_SETTING = 'autohide-sidebars'
export const HIDE_OST = 'hide-ost'
export const SHOW_ALBUM_TAGS = 'show-album-tags'
export const SHOW_ARTIST_TAGS = 'show-artist-tags'

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

/* TAGS */
/**
 * The tags store contains all of the user-defined tags, alongside
 * information on what tracks they belong to.
 */
export const TAGS_FILENAME = 'tags.json'
export const TAGS_SETTING = 'tags'

/* DISCORD */
/**
 * The Discord store contains data that is shared between the frontend and
 * the backend. Strictly speaking, it does not need to be persistent, as it is
 * regenerated every time the bot connects, but it works well as a debugging aid.
 */
export const DISCORD_FILENAME = 'discord.json'
export const GUILDS_SETTING = 'guilds'

/* SVELTE STATE */
export type AppState = {
    guildId: number
    player: Player
    offline: boolean
    playlist: Playlist
    recentlyPlayed: Track[]
    parallel: Parallel
    tagsMode: 'all' | 'any'
    trackSearchTerm: string
}

/**
 * The global state of the application, containing info on available tracks,
 * the main player, the main queue, parallel tracks and their player and
 * recent tracks.
 */
export const appState: AppState = $state({
    guildId: 0,
    offline: true,
    player: new Player({
        playing: false,
        position: 0,
        volume: 0.5,
        loopState: LoopState.None,
        shuffle: false,
        mute: false
    }),
    playlist: new Playlist([], [], []),
    recentlyPlayed: [],
    parallel: new Parallel([]),
    tagsMode: 'all',
    trackSearchTerm: ''
})

/**
 * The set of tags and groups that the entire app should have access to.
 */
export const appTags = new TagGroupSet([])

/**
 * The set of currently selected tags.
 */
export const selectedTags = new TagSet([])

/**
 * All the tracks from the active audio sources alongside their visibility state.
 */
export const appTracks = new TrackSet([])

export type Settings = {
    darkMode: boolean
    showCovers: boolean
    autoconnect: boolean
    autohideSidebars: boolean
    hideOst: boolean
    showAlbumTags: boolean
    showArtistTags: boolean
}

/**
 * The settings for Bardic Lore. Initialize these in `layout.ts`.
 */
export const settings: Settings = $state({
    darkMode: true,
    showCovers: true,
    autoconnect: false,
    autohideSidebars: true,
    hideOst: true,
    showAlbumTags: false,
    showArtistTags: false
})

/**
 * Used to skip removing the current track on the next TRACK_ENDED signals.
 * Each TRACK_ENDED event reduces this counter by one. Mostly useful for backskips
 * and to avoid unnecessary processing when resetting a playlist.
 */
export const skipRemoveOnEnd = $state({ toSkip: 0 })

/**
 * Stores cover image blob URLs so that they can be shared between images.
 * Prevents reading the image files every time there is a track with a cover.
 * The covers are 200x200 pixels. They are encoded as WebP.
 */
export const cachedCoverImages: SvelteMap<string, string> = new SvelteMap()
/**
 * Stores cover image blob URLs so that they can be shared between images.
 * Prevents reading the image files every time there is a track with a cover.
 * The thumbnails are 64x64 pixels and should be used for icon-like use cases.
 * They are encoded as WebP.
 */
export const cachedCoverThumbnails: SvelteMap<string, string> = new SvelteMap()

/**
 * The top offset of the central virtual list, mainly used to fix the context menu
 * coordinates on a right click.
 */
export const virtualListTop = $state({ top: 0 })
