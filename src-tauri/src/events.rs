//! This module contains events emitted by Tauri for communication, primarily
//! between the Discord bot and the UI to keep them synchronized.

// All events should be written in kebab-case.
// This file should be synchronized with src/lib/events.ts in the frontend.
// If you need to add or change an event, please change the TypeScript file too.

/* FROM UI TO BOT */
/// This event tells the Bot to enter a voice channel.
/// The payload must include the guild ID and the channel ID.
pub const JOIN_VOICE_CHANNEL: &str = "join-voice-channel";
/// This event tells the Bot to leave the voice channel
/// it is currently in. The payload must include the guild ID.
pub const LEAVE_VOICE_CHANNEL: &str = "leave-voice-channel";
/// This event tells the bot to add a track to the queue.
/// The payload must include the filepath of the file to play and
/// the guild ID.
pub const QUEUE_TRACK: &str = "queue-track";
/// This event tells the bot to resume playback of its queue.
/// If playback is not paused or the queue is empty, it does nothing.
/// The payload must include the guild ID.
pub const RESUME_PLAYBACK: &str = "resume-playback";
/// This event tells the bot to pause playback of its queue.
/// If playback is not playing or the queue is empty, it does nothing.
/// The payload must include the guild ID.
pub const PAUSE_PLAYBACK: &str = "pause-playback";
/// This event tells the bot to skip the current track in the queue.
/// The payload must include the guild ID.
pub const SKIP_TRACK: &str = "skip-track";
/// This event tells the bot to loop the current track indefinitely.
/// The payload must include the guild ID.
pub const LOOP_TRACK: &str = "loop-track";

/* FROM BOT TO UI */
/// This event indicates that there was an error in a bot command. It is
/// generic and applies for any error related to the serenity client.
/// The payload should contain more information.
pub const BOT_ERROR: &str = "bot-error";
/// This event indicates that the bot received information on a new guild
/// and the Tauri store was update to match.
/// It is fired whenever the bot receives a GUILD_CREATE event.
pub const UPDATED_GUILDS: &str = "updated-guilds";
/// This event instructs the frontend to update the current track $state rune
/// using the information passed in the payload. The payload must be a `serde_json`
/// `Value`, probably made with the `json!` macro. The frontend will update the
/// fields in the $state based on which keys match. See src/lib/stores.svelte.ts
/// for the data structure. Make sure the types are correct.
pub const UPDATE_TRACK: &str = "update-track";
/// This event notifies the frontend that a track just finished. Essentially a relay
/// of songbird's `TrackEvent::End`.
pub const TRACK_ENDED: &str = "track-ended";
