// This file contains events emitted by Tauri for communication, primarily
// between the Discord bot and the UI to keep them synchronized.

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
 * The payload must include the filepath of the file to play and
 * the guild ID.
 */
export const QUEUE_TRACK = 'queue-track'
/**
 * This event tells the bot to play a track on top of existing ones.
 * It will not be added to the queue. The payload must include the guild ID
 * and a Track object.
 */
export const PLAY_PARALLEL = 'play-parallel'
/**
 * This event tells the bot to resume playback of its queue.
 * If playback is not paused or the queue is empty, it does nothing.
 * The payload must include the guild ID.
 */
export const RESUME_PLAYBACK = 'resume-playback'
/**
 * This event tells the bot to pause playback of its queue.
 * If playback is not playing or the queue is empty, it does nothing.
 * The payload must include the guild ID.
 */
export const PAUSE_PLAYBACK = 'pause-playback'
/**
 * This event tells the bot to skip the current track in the queue.
 * The payload must include the guild ID.
 */
export const SKIP_TRACK = 'skip-track'
/**
 * This event tells the bot to loop the current track indefinitely.
 * The payload must include the guild ID.
 */
export const LOOP_TRACK = 'loop-track'
/**
 * This event tells the bot to seek to the given position.
 * The payload must include the guild ID and the position as a u64.
 */
export const SEEK_TRACK = 'seek-track'
/**
 * This event tells the bot to change the playback volume.
 * The payload must include the guild ID and volume as an f32.
 */
export const CHANGE_VOLUME = 'change-volume'
/**
 * This event tells the bot to mute or unmute, inverting the state.
 * The payload must include the guild ID.
 */
export const MUTE_UNMUTE = 'mute-unmute'

/* FROM BOT TO UI */
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
 * This event instructs the frontend to update the playerState $state rune
 * using the information passed in the payload. The payload must be a `serde_json`
 * `Value`, probably made with the `json!` macro. The frontend will update the
 * fields in the $state based on which keys match. See src/lib/stores.svelte.ts
 * for the data structure. Make sure the types are correct.
 */
export const UPDATE_PLAYER = 'update-player'

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
