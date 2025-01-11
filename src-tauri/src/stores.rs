//! This module contains constants to access the Tauri stores that Bardic Lore
//! uses to keep things synchronized between the front- and backend and also for
//! persistent storage.

// All keys should be written in kebab-case.
// This file should be synchronized with src/lib/stores.svelte.ts in the frontend.
// If you need to add or change a store, please change the TypeScript file too.

/* SETTINGS */
/// The settings store contains all user settings, such as audio sources
/// and the Discord bot token.
pub const SETTINGS_FILENAME: &str = "settings.json";
pub const AUDIO_SOURCES_SETTING: &str = "audio-sources";
pub const BOT_TOKEN_SETTING: &str = "bot-token";
pub const VOLUME_SETTING: &str = "volume";

/* TRACKS */
/// The tracks store is a cache for the tracks found in the audio sources.
/// Tracks should be written to this store once when the sources change and
/// then all access to tracks should be from here. This avoids re-reading
/// possibly hundreds of tracks with attached metadata everytime the app
/// starts.
pub const TRACKS_FILENAME: &str = "tracks.json";
pub const TRACKS_SETTING: &str = "tracks";

/* DISCORD */
/// The Discord store contains data that is shared between the frontend and
/// the backend. Strictly speaking, it does not need to be persistent, as it is
/// regenerated every time the bot connects, but it works well as a debugging aid.
pub const DISCORD_FILENAME: &str = "discord.json";
pub const GUILDS_SETTING: &str = "guilds";
