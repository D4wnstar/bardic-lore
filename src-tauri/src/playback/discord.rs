use std::{collections::HashSet, str::FromStr, sync::Arc, time::Duration};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serenity::{
    all::{ChannelId, ChannelType, Context, EventHandler, GatewayIntents, Guild, GuildId, Ready},
    async_trait,
    prelude::TypeMapKey,
};

use songbird::{
    events::EventData,
    tracks::{LoopState, Track},
    EventContext, EventHandler as VoiceEventHandler, SerenityInit, Songbird, TrackEvent,
};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_store::StoreExt;
use tokio::{sync::Mutex as AsyncMutex, time::sleep};
use tracing::debug;
use uuid::Uuid;

use crate::{
    events::{
        CreatePlaylistPayload, GuildChannelIdPayload, GuildIdPayload, PlayParallelPayload,
        QueueMethod, QueueTrackPayload, TrackActionPayload, ADD_TRACK, BOT_ERROR, CLIENT_CONNECTED,
        CREATE_PLAYLIST, JOIN_VOICE_CHANNEL, LEAVE_VOICE_CHANNEL, LEFT_VOICE_CHANNEL, MUTE_UNMUTE,
        PLAYLIST_CREATED, PLAY_PARALLEL, QUEUE_EMPTIED, QUEUE_SORTED, QUEUE_TRACK, TRACK_ENDED,
        TRACK_LOOPED, TRACK_PAUSED, TRACK_PLAYABLE, TRACK_PLAYED, UPDATED_GUILDS, UPDATE_PLAYER,
        UPDATE_TRACKS,
    },
    playback::parallel::ParallelTracks,
    playback::queue::TrackQueue,
    stores::{BOT_TOKEN_SETTING, DISCORD_FILENAME, GUILDS_SETTING, SETTINGS_FILENAME},
    Error,
};

/// Wrapper struct to check if a serenity client already exists
pub struct IsSerenityClientOn(pub bool);

#[derive(Serialize, Deserialize, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct GuildSlug {
    id: GuildId,
    name: String,
    voice_channels: Vec<VoiceChannelSlug>,
}

impl PartialEq for GuildSlug {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct VoiceChannelSlug {
    id: ChannelId,
    name: String,
    active: bool,
}

impl PartialEq for VoiceChannelSlug {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

struct QueueKey;

impl TypeMapKey for QueueKey {
    type Value = TrackQueue;
}

struct ParallelKey;

impl TypeMapKey for ParallelKey {
    type Value = ParallelTracks;
}

pub struct Handler {
    app: AppHandle,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _ready: Ready) {
        self.app.emit(CLIENT_CONNECTED, ()).unwrap();

        let client_exists_mutex = self.app.state::<AsyncMutex<IsSerenityClientOn>>();
        let mut client_exists = client_exists_mutex.lock().await;
        client_exists.0 = true;
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        // serenity has no API to tell the bot to do something from code,
        // it can only handle Gateway events sent by Discord, such as chat messages
        // We need to control the bot manually from the frontend though, so we
        // setup Tauri event listeners instead. Whenever we want the bot to do something on
        // command, we register a callback for an event right here and then emit that
        // event from anywhere

        let manager = songbird::get(&ctx)
            .await
            .expect("Failed to get Songbird manager");
        let ctx = Arc::new(ctx);

        // Each callback needs to have ownership of whatever it uses since it outlives this functions
        // The need for a tokio::spawn to permit await calls causes annoying double-cloning of Arcs
        // because they need to be moved twice (first in the callback, then in the tokio async task)
        // Performance isn't a concern for these callbacks, but it's just kind of ugly
        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(JOIN_VOICE_CHANNEL, move |ev| {
            let (manager, app, _) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move {
                join_voice_channel(ev, &manager, &app).await;
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(LEAVE_VOICE_CHANNEL, move |ev| {
            let (manager, app, _) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move {
                leave_voice_channel(ev, &manager, &app).await;
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(CREATE_PLAYLIST, move |ev| {
            let (manager, app, ctx) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move {
                create_playlist(ev, &manager, &app, &ctx).await;
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(QUEUE_TRACK, move |ev| {
            let (manager, app, ctx) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move {
                queue_track(ev, &manager, &app, &ctx).await;
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(PLAY_PARALLEL, move |ev| {
            let (manager, app, ctx) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move {
                play_parallel(ev, &manager, &app, &ctx).await;
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(UPDATE_TRACKS, move |ev| {
            let (manager, app, ctx) = clone_boilerplate(&manager1, &app1, &ctx1);
            let payload: TrackActionPayload = serde_json::from_str(ev.payload()).unwrap();
            tokio::spawn(async move {
                if payload.parallel {
                    parallel_action(payload, &manager, &app, &ctx).await;
                } else {
                    queue_action(payload, &manager, &app, &ctx).await;
                }
            });
        });

        let (manager1, app1, ctx1) = clone_boilerplate(&manager, &self.app, &ctx);
        self.app.listen(MUTE_UNMUTE, move |ev| {
            let (manager, app, _) = clone_boilerplate(&manager1, &app1, &ctx1);
            tokio::spawn(async move { mute_unmute(ev, &manager, &app).await });
        });
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        let voice_channels: Vec<VoiceChannelSlug> = guild
            .channels
            .iter()
            .filter_map(|(id, channel)| {
                if channel.kind == ChannelType::Voice {
                    return Some(VoiceChannelSlug {
                        id: id.clone(),
                        name: channel.name.clone(),
                        active: false,
                    });
                } else {
                    return None;
                }
            })
            .collect();

        if let Ok(store) = self.app.store(DISCORD_FILENAME) {
            let guilds_value = store.get(GUILDS_SETTING).unwrap_or(json!([]));
            let mut guilds: HashSet<GuildSlug> =
                serde_json::from_value(guilds_value).unwrap_or(HashSet::new());
            guilds.insert(GuildSlug {
                id: guild.id,
                name: guild.name,
                voice_channels,
            });
            if let Ok(value) = serde_json::to_value(guilds) {
                store.set(GUILDS_SETTING, value);
                self.app.emit(UPDATED_GUILDS, ()).unwrap_or_else(|why| {
                        eprintln!("Failed to send updated-guilds event. Please refresh guilds manually. Reason: {why}")
                    });
            }
        } else {
            eprintln!(
                "Failed to get AppHandle from client context data. Guilds have not been updated"
            )
        }
    }

    // TODO: Also update on guild delete
}

/* TAURI COMMANDS */
#[tauri::command]
pub async fn create_discord_client(app: AppHandle) -> Result<(), Error> {
    let client_exists_mutex = app.state::<AsyncMutex<IsSerenityClientOn>>();
    let client_exists = client_exists_mutex.lock().await;
    if client_exists.0 {
        return Err(Error::SerenityClientAlreadyExists());
    }

    // Get token from store
    let store = app.store(SETTINGS_FILENAME)?;
    let value = store.get(BOT_TOKEN_SETTING).unwrap_or("".into());
    let token: String = serde_json::from_value(value)?;
    let mut ds_client = serenity::Client::builder(token, GatewayIntents::non_privileged())
        .event_handler(Handler { app: app.clone() })
        .type_map_insert::<ParallelKey>(ParallelTracks::new())
        .type_map_insert::<QueueKey>(TrackQueue::new())
        .register_songbird()
        .await?;

    let async_app = app.clone();
    tokio::spawn(async move {
        if let Err(why) = ds_client.start().await {
            print_emit_error(
                BOT_ERROR,
                &format!("Failed to connect to Discord. Error: {why}"),
                &async_app,
            );
        }
    });

    return Ok(());
}

#[tauri::command]
pub async fn is_bot_connected(app: AppHandle) -> bool {
    let client_exists_mutex = app.state::<AsyncMutex<IsSerenityClientOn>>();
    return client_exists_mutex.lock().await.0;
}

/* EVENT CALLBACKS */
async fn join_voice_channel(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle) {
    let payload: GuildChannelIdPayload = serde_json::from_str(ev.payload()).unwrap();
    // Every time you join a channel, you also leave the previous one
    // This is emitted BEFORE the join call since join is lazy and runs
    // at the same time as the first track being queued, which leads to the
    // track being removed instantly due to the LEFT_VOICE_CHANNEL event if
    // we run it after
    app.emit(LEFT_VOICE_CHANNEL, ()).unwrap();
    if let Err(err) = manager.join(payload.guildId, payload.channelId).await {
        print_emit_error(BOT_ERROR, &format!("Failed to join channel. {err}"), &app);
    };
}

async fn leave_voice_channel(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle) {
    let payload: GuildIdPayload = serde_json::from_str(ev.payload()).unwrap();
    if manager.get(payload.guildId).is_some() {
        manager
            .remove(payload.guildId)
            .await
            .expect("Failed to leave channel");

        app.emit(LEFT_VOICE_CHANNEL, ()).unwrap();
    } else {
        print_emit_error(BOT_ERROR, "Bot not in a voice channel", &app);
    };
}

async fn create_playlist(
    ev: tauri::Event,
    manager: &Arc<Songbird>,
    app: &AppHandle,
    ctx: &Context,
) {
    let payload: CreatePlaylistPayload = serde_json::from_str(ev.payload())
        .inspect_err(|_e| {
            tracing::error!("Failed to parse CreatePlaylistPayload: {:#?}", ev.payload())
        })
        .unwrap();

    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    }
    if payload.tracksData.len() == 0 {
        print_emit_error(BOT_ERROR, "The given array of tracks is empty", &app);
        return;
    }

    let handler_lock = maybe_handler.unwrap();
    let mut handler = handler_lock.lock().await;
    let data = ctx.data.read().await;
    let queue = data.get::<QueueKey>().expect("Guaranteed to exist");

    queue.stop();
    // Wait a few milliseconds to let the frontend process the events fired by stopping the queue
    // sleep(Duration::from_millis(50)).await;
    // app.emit(QUEUE_EMPTIED, ()).unwrap();

    let mut tracks_data = payload.tracksData;
    if payload.shuffle {
        // Index 0 guaranteed because of the previous if let
        // Also, the fact that this doesn't preserve order is just added randomness
        let first_track = tracks_data.swap_remove(0);
        // TODO: Move the RNG creation somewhere to improve randomness
        tracks_data.shuffle(&mut rand::thread_rng());
        tracks_data.insert(0, first_track);
    };

    let mut response_tracks = vec![];
    for (idx, track_data) in tracks_data.iter().enumerate() {
        let mut track: Track = songbird::input::File::new(track_data.path.clone()).into();

        track.volume = payload.volume;
        if idx == 0 && payload.loopFirst {
            track = track.loops(LoopState::Infinite);
        }
        add_trackevent_relays(&app, &mut track, false);
        let uuid = track.uuid.clone();

        queue.add(track, &mut handler).await;

        response_tracks.push(json!({
            "uuid": uuid,
            "title": track_data.title,
            "album": track_data.album,
            "artist": track_data.artist,
            "duration": track_data.duration,
            "path": track_data.path,
            "filename": track_data.filename,
            "coverHash": track_data.cover_hash,
        }));
    }
    let out = serde_json::to_value(&response_tracks).unwrap();
    let res = json!({ "tracks": out });
    // Sleep for a short time to guarantee that the QUEUE_EMPTIED event will be processed first
    sleep(Duration::from_millis(50)).await;
    app.emit(PLAYLIST_CREATED, res).unwrap();
}

async fn queue_track(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle, ctx: &Context) {
    let payload: QueueTrackPayload = serde_json::from_str(ev.payload())
        .inspect_err(|_e| tracing::error!("Failed to parse QueueTrackPayload: {:#?}", ev.payload()))
        .unwrap();
    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    }

    let handler_lock = maybe_handler.unwrap();
    let mut handler = handler_lock.lock().await;
    let data = ctx.data.read().await;
    let queue = data.get::<QueueKey>().expect("Guaranteed to exist");
    let mut track: Track = songbird::input::File::new(payload.trackData.path.clone()).into();

    track.volume = payload.volume;
    if payload.looping {
        track = track.loops(LoopState::Infinite);
    }
    add_trackevent_relays(&app, &mut track, false);
    let uuid = track.uuid.clone();

    let response = match payload.queueMethod {
        QueueMethod::Normal => {
            queue.add(track, &mut handler).await;
            json!({})
        }
        QueueMethod::OverwriteCurrent => {
            if queue.is_empty() {
                queue.add(track, &mut handler).await;
            } else {
                queue.overwrite_current(track, &mut handler).await;
            }
            json!({ "position": 0 })
        }
        QueueMethod::Priority => {
            queue.add_priority(track, &mut handler).await;
            json!({})
        }
        QueueMethod::Backskip => {
            queue.prepend(track, &mut handler).await;
            json!({ "position": 0 })
        }
    };

    debug!("Emitting {UPDATE_PLAYER} event");
    app.emit(UPDATE_PLAYER, response).unwrap();
    debug!("Emitting {ADD_TRACK} event");
    app.emit(
        ADD_TRACK,
        json!({
            "track": {
                "uuid": uuid,
                "title": payload.trackData.title,
                "album": payload.trackData.album,
                "artist": payload.trackData.artist,
                "duration": payload.trackData.duration,
                "path": payload.trackData.path,
                "filename": payload.trackData.filename,
                "coverHash": payload.trackData.cover_hash,
            },
            "parallel": false,
            "queueMethod": payload.queueMethod,
            "looping": payload.looping,
        }),
    )
    .unwrap();
}

async fn play_parallel(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle, ctx: &Context) {
    let payload: PlayParallelPayload = serde_json::from_str(ev.payload()).unwrap();
    if let Some(handler_lock) = manager.get(payload.guildId) {
        let mut handler = handler_lock.lock().await;
        let data = ctx.data.read().await;
        let parallel = data.get::<ParallelKey>().expect("Guaranteed to exist");

        let mut track: Track = songbird::input::File::new(payload.trackData.path.clone()).into();
        track.volume = payload.volume;
        if payload.looping {
            track = track.loops(LoopState::Infinite);
        }
        add_trackevent_relays(&app, &mut track, true);
        let uuid = track.uuid.clone();

        parallel.add(track, &mut handler);

        debug!("Emitting {ADD_TRACK} event");
        app.emit(
            ADD_TRACK,
            json!({
                "track": {
                    "uuid": uuid,
                    "title": payload.trackData.title,
                    "album": payload.trackData.album,
                    "artist": payload.trackData.artist,
                    "duration": payload.trackData.duration,
                    "path": payload.trackData.path,
                    "filename": payload.trackData.filename,
                    "cover_hash": payload.trackData.cover_hash,
                },
                "parallel": true,
                "overwrite": false,
                "prepend": false,
                "looping": payload.looping,
            }),
        )
        .unwrap();
    } else {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
    };
}

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum TrackAction {
    Resume,
    Pause,
    Skip,
    Stop,
    Loop,
    Seek,
    ChangeVolume,
    Shuffle,
    Sort,
}

async fn queue_action(
    payload: TrackActionPayload,
    manager: &Arc<Songbird>,
    app: &AppHandle,
    ctx: &Context,
) {
    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    };

    let data = ctx.data.read().await;
    let queue = data.get::<QueueKey>().expect("Guaranteed to exist");

    let action = payload.action;

    let msg_str = match action {
        TrackAction::Resume => "resume",
        TrackAction::Pause => "pause",
        TrackAction::Skip => "skip",
        TrackAction::Stop => "stop",
        TrackAction::Loop => "activate loop for",
        TrackAction::Seek => "seek",
        TrackAction::ChangeVolume => "change volume",
        TrackAction::Shuffle => "shuffle",
        TrackAction::Sort => "sort",
    };
    // Most of these are handled automatically by the TrackEvent relays
    let response_event = match action {
        TrackAction::Resume => None,
        TrackAction::Pause => None,
        TrackAction::Skip => None,
        TrackAction::Stop => None,
        TrackAction::Loop => None,
        TrackAction::Seek => Some(UPDATE_PLAYER),
        TrackAction::ChangeVolume => None,
        TrackAction::Shuffle => Some(QUEUE_SORTED),
        TrackAction::Sort => Some(QUEUE_SORTED),
    };

    let result = match action {
        TrackAction::Resume => (Some(queue.resume()), None),
        TrackAction::Pause => (Some(queue.pause()), None),
        TrackAction::Skip => (Some(queue.skip()), None),
        TrackAction::Stop => {
            queue.stop();
            (None, None)
        }
        TrackAction::Loop => {
            if let Some(track) = queue.current() {
                let maybe_info = track.get_info().await;
                if let Err(err) = maybe_info {
                    print_emit_error(
                        BOT_ERROR,
                        &format!("Failed to get info for current track. {err}"),
                        app,
                    );
                    return;
                };
                let res = match maybe_info.unwrap().loops {
                    LoopState::Infinite => track.disable_loop(),
                    LoopState::Finite(_) => track.enable_loop(),
                };
                (Some(res), None)
            } else {
                (None, None)
            }
        }
        TrackAction::Seek => {
            if let Some(curr_track) = queue.current() {
                if let Some(position) = payload.position {
                    let time = Duration::from_secs(position);
                    let res = curr_track.seek_async(time).await;
                    // Drop the duration inside to avoid a type mismatch with other arms
                    let res = res.map(|_| ());
                    (Some(res), Some(json!({ "position": position })))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
        TrackAction::ChangeVolume => {
            let volume = payload
                .volume
                .expect("Volume change event should have volume as payload");
            queue.modify_queue(|q| {
                for track in q {
                    track
                        .set_volume(volume)
                        .expect("Couldn't change volume on track");
                }
            });
            (None, None)
        }
        TrackAction::Shuffle => {
            queue.shuffle(&mut rand::thread_rng());
            let uuids: Vec<Uuid> = queue
                .current_queue()
                .iter()
                .map(|handle| handle.uuid())
                .collect();
            (None, Some(json!({ "uuids": uuids })))
        }
        TrackAction::Sort => {
            let uuids = payload
                .sortUuids
                .expect("Sort action should have UUIDs to sort by")
                .iter()
                .map(|str| Uuid::from_str(&str).unwrap())
                .collect();
            queue.sort_by_uuids(uuids);
            // These should be identical to the given uuids, but we recollect
            // them just in case something went wrong during sorting
            let new_uuids: Vec<Uuid> = queue
                .current_queue()
                .iter()
                .map(|handle| handle.uuid())
                .collect();
            (None, Some(json!({ "uuids": new_uuids })))
        }
    };

    match result {
        (Some(Err(err)), _) => {
            print_emit_error(
                BOT_ERROR,
                &format!("Failed to {msg_str} track. {err}"),
                &app,
            );
        }
        (_, Some(payload)) => {
            if let Some(event) = response_event {
                app.emit(event, payload).unwrap();
            }
        }
        _ => {
            if let Some(event) = response_event {
                app.emit(event, ()).unwrap();
            }
        }
    }
}

async fn parallel_action(
    payload: TrackActionPayload,
    manager: &Arc<Songbird>,
    app: &AppHandle,
    ctx: &Context,
) {
    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    };
    let uuid = Uuid::parse_str(
        &payload
            .uuid
            .expect("UUID must be present to modify parallel track"),
    )
    .unwrap();

    // let lock = maybe_handler.unwrap();
    // let handler = lock.lock().await;

    let data = ctx.data.read().await;
    let parallel = data.get::<ParallelKey>().expect("Guaranteed to exist");

    let action = payload.action;

    let payload = match action {
        TrackAction::Resume => {
            parallel.resume(uuid);
            None
        }
        TrackAction::Pause => {
            parallel.pause(uuid);
            None
        }
        TrackAction::Stop => {
            parallel.stop(uuid);
            None
        }
        TrackAction::Loop => {
            if let Some(handle) = parallel.get_handle(uuid) {
                if let Ok(info) = handle.get_info().await {
                    drop(match info.loops {
                        LoopState::Infinite => handle.disable_loop(),
                        LoopState::Finite(_) => handle.enable_loop(),
                    });
                }
            };
            None
        }
        TrackAction::Seek => {
            let position = Duration::from_secs(payload.position.unwrap());
            if let Some(track) = parallel.get_handle(uuid) {
                drop(track.seek_async(position).await);
            };
            let seconds = position.as_secs();
            Some(json!({ "position": seconds, "uuid": uuid }))
        }
        TrackAction::ChangeVolume => {
            let volume = payload
                .volume
                .expect("Volume change event should have volume as payload");
            if let Some(track) = parallel.get_handle(uuid) {
                drop(track.set_volume(volume));
            };
            None
        }
        _ => None,
    };

    if let Some(payload) = payload {
        app.emit(UPDATE_PLAYER, payload).unwrap();
    }
}

async fn mute_unmute(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle) {
    let payload: GuildIdPayload = serde_json::from_str(ev.payload()).unwrap();
    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    };
    let lock = maybe_handler.unwrap();
    let mut handler = lock.lock().await;
    let is_mute = handler.is_mute();
    let res = handler.mute(!is_mute).await;
    if let Err(err) = res {
        print_emit_error(
            BOT_ERROR,
            &format!("Error when changing mute state: {err}"),
            &app,
        );
    } else {
        app.emit(UPDATE_PLAYER, json!({ "mute": !is_mute }))
            .unwrap();
    }
}

/* CONVENIENCE FUNCTIONS */
/// Print an error to stderr and emit a Tauri event with the same message as the payload.
fn print_emit_error(ev_name: &str, error_msg: &str, app: &AppHandle) {
    tracing::error!("{error_msg}");
    app.emit(ev_name, format!("{error_msg}")).unwrap();
}

/// Clone some references to have them moved into callbacks/tasks.
fn clone_boilerplate(
    manager: &Arc<Songbird>,
    app: &AppHandle,
    ctx: &Arc<Context>,
) -> (Arc<Songbird>, AppHandle, Arc<Context>) {
    let manager1 = Arc::clone(manager);
    let app1 = app.clone();
    let ctx1 = Arc::clone(ctx);
    return (manager1, app1, ctx1);
}

/// Add event handlers to a `Track` so that `TrackEvents` also fire Tauri events.
/// This allows the rest of the Tauri app to "see" the `TrackEvents` in real time.
/// These are not added as global events because only main queue tracks should fire
/// these. Parallel tracks behave differently.
fn add_trackevent_relays(app: &AppHandle, track: &mut Track, is_parallel: bool) {
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::End),
            RelayTrackEnd {
                app: app.clone(),
                is_parallel,
                uuid: track.uuid.to_string(),
            },
        ),
        Duration::ZERO,
    );
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::Loop),
            RelayTrackLoop {
                app: app.clone(),
                is_parallel,
                uuid: track.uuid.to_string(),
            },
        ),
        Duration::ZERO,
    );
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::Playable),
            RelayTrackPlayable {
                app: app.clone(),
                is_parallel,
                uuid: track.uuid.to_string(),
            },
        ),
        Duration::ZERO,
    );
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::Play),
            RelayTrackPlay {
                app: app.clone(),
                is_parallel,
                uuid: track.uuid.to_string(),
            },
        ),
        Duration::ZERO,
    );
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::Pause),
            RelayTrackPause {
                app: app.clone(),
                is_parallel,
                uuid: track.uuid.to_string(),
            },
        ),
        Duration::ZERO,
    );
}

/* EVENT HANDLERS */
// The common JSON schema for all of these is defined in src/lib/events.ts in the frontend
struct RelayTrackEnd {
    app: AppHandle,
    is_parallel: bool,
    uuid: String,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackEnd {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_ENDED,
                json!({ "isParallel": self.is_parallel, "uuid": self.uuid }),
            )
            .expect(&format!("Couldn't emit {TRACK_ENDED} event"));
        return None;
    }
}

struct RelayTrackLoop {
    app: AppHandle,
    is_parallel: bool,
    uuid: String,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackLoop {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_LOOPED,
                json!({ "isParallel": self.is_parallel, "uuid": self.uuid }),
            )
            .expect(&format!("Couldn't emit {TRACK_LOOPED} event"));
        return None;
    }
}

struct RelayTrackPlayable {
    app: AppHandle,
    is_parallel: bool,
    uuid: String,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPlayable {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PLAYABLE,
                json!({ "isParallel": self.is_parallel, "uuid": self.uuid }),
            )
            .expect(&format!("Couldn't emit {TRACK_PLAYABLE} event"));
        return None;
    }
}

struct RelayTrackPlay {
    app: AppHandle,
    is_parallel: bool,
    uuid: String,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPlay {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PLAYED,
                json!({ "isParallel": self.is_parallel, "uuid": self.uuid }),
            )
            .expect(&format!("Couldn't emit {TRACK_PLAYED} event"));
        return None;
    }
}

struct RelayTrackPause {
    app: AppHandle,
    is_parallel: bool,
    uuid: String,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPause {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PAUSED,
                json!({ "isParallel": self.is_parallel, "uuid": self.uuid }),
            )
            .expect(&format!("Couldn't emit {TRACK_PAUSED} event"));
        return None;
    }
}
