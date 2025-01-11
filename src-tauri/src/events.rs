#![allow(non_snake_case)] // Payloads are in JSON so camelCase it is

//! This module contains events emitted by Tauri for communication, primarily
//! between the Discord bot and the UI to keep them synchronized.

// All events should be written in kebab-case.
// This file should be synchronized with src/lib/events.ts in the frontend.
// If you need to add or change an event, please change the TypeScript file too.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serenity::all::{ChannelId, GuildId};

use crate::files::Track;

/* FROM UI TO BOT */
/// This event tells the Bot to enter a voice channel.
/// The payload must include the guild ID and the channel ID.
pub const JOIN_VOICE_CHANNEL: &str = "join-voice-channel";
/// This event tells the Bot to leave the voice channel
/// it is currently in. The payload must include the guild ID.
pub const LEAVE_VOICE_CHANNEL: &str = "leave-voice-channel";
/// This event tells the bot to add a track to the queue.
/// The payload must include the guild ID and a Track object.
pub const QUEUE_TRACK: &str = "queue-track";
/// This event tells the bot to play a track on top of existing ones.
/// It will not be added to the queue. The payload must include the guild ID
/// and a Track object.
pub const PLAY_PARALLEL: &str = "play-parallel";
/// This event tells the bot to resume playback of its queue.
/// If playback is not paused or the queue is empty, it does nothing.
/// The payload must include the guild ID.
pub const RESUME_PLAYBACK: &str = "resume-playback";
/// This event tells the bot to pause playback of its queue.
/// If playback is not playing or the queue is empty, it does nothing.
/// The payload must include the guild ID.
pub const PAUSE_PLAYBACK: &str = "pause-playback";
/// This event tells the bot to stop the current track and delete the queue,
/// unless it is a parallel track, in which case it only stops that track.
/// The payload must include the guild ID.
pub const STOP_TRACK: &str = "stop-track";
/// This event tells the bot to skip the current track in the queue.
/// The payload must include the guild ID.
pub const SKIP_TRACK: &str = "skip-track";
/// This event tells the bot to loop the current track indefinitely.
/// The payload must include the guild ID.
pub const LOOP_TRACK: &str = "loop-track";
/// This event tells the bot to seek to the given position.
/// The payload must include the guild ID and the position as a u64.
pub const SEEK_TRACK: &str = "seek-track";
/// This event tells the bot to change the playback volume.
/// The payload must include the guild ID and volume as an f32.
pub const CHANGE_VOLUME: &str = "change-volume";
/// This event tells the bot to mute or unmute, inverting the state.
/// The payload must include the guild ID.
pub const MUTE_UNMUTE: &str = "mute-unmute";

/* FROM BOT TO UI */
/// This event indicates that there was an error in a bot command. It is
/// generic and applies for any error related to the serenity client.
/// The payload should contain more information.
pub const BOT_ERROR: &str = "bot-error";
/// This event indicates that the bot received information on a new guild
/// and the Tauri store was update to match.
/// It is fired whenever the bot receives a GUILD_CREATE event.
pub const UPDATED_GUILDS: &str = "updated-guilds";
/// This event indicates that the bot left its voice channel and either joined
/// a new one or it went offline.
pub const LEFT_VOICE_CHANNEL: &str = "left-voice-channel";
/// This event instructs the frontend to update something related to the global
/// appState using the information passed in the payload. The payload must be
/// a `serde_json` `Value`, probably made with the `json!` macro. See
/// src/lib/stores.svelte.ts for the data structure. Make sure the types are correct.
pub const UPDATE_PLAYER: &str = "update-player";
/// This event instructs the frontend to add the track given in the payload to
/// either the main queue or the parallel tracks, depending on what the payload says.
/// By track here we mean a `crate::files::Track`, not a serenity `Track`.
pub const ADD_TRACK: &str = "add-track";
/// This event instructs the frontend to clear the whole queue and stop playback.
pub const CLEAR_QUEUE: &str = "clear-queue";

/* TRACKEVENT RELAYS */
/// This event notifies the frontend that a track just finished. Essentially a relay
/// of songbird's `TrackEvent::End`.
pub const TRACK_ENDED: &str = "track-ended";
/// This event notifies the frontend that a track just looped. Essentially a relay
/// of songbird's `TrackEvent::Loop`.
pub const TRACK_LOOPED: &str = "track-looped";
/// This event notifies the frontend that a track just became playable. Essentially a relay
/// of songbird's `TrackEvent::Playable`.
pub const TRACK_PLAYABLE: &str = "track-playable";
/// This event notifies the frontend that a track just resumed playing. Essentially a relay
/// of songbird's `TrackEvent::Play`.
pub const TRACK_PLAYED: &str = "track-played";
/// This event notifies the frontend that a track just paused. Essentially a relay
/// of songbird's `TrackEvent::Pause`.
pub const TRACK_PAUSED: &str = "track-paused";

/* PAYLOADS */
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildIdPayload {
    pub guildId: GuildId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildChannelIdPayload {
    pub guildId: GuildId,
    pub channelId: ChannelId,
}

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum QueueMethod {
    Normal,
    Priority,
    Prepend,
    OverwriteCurrent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueTrackPayload {
    pub guildId: GuildId,
    pub trackData: Track,
    pub looping: bool,
    pub queueMethod: QueueMethod,
    pub volume: f32,
    pub numberOfPriority: Option<u64>,
    pub isPriorityPlaying: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayParallelPayload {
    pub guildId: GuildId,
    pub trackData: Track,
    pub looping: bool,
    pub volume: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackActionPayload {
    pub guildId: GuildId,
    pub parallel: bool,
    pub uuid: Option<String>,
    pub position: Option<u64>,
    pub volume: Option<f32>,
}
