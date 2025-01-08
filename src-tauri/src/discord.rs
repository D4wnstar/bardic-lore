use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    all::{ChannelId, ChannelType, Context, EventHandler, GatewayIntents, Guild, GuildId},
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
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    events::{
        GuildChannelIdPayload, GuildIdPayload, PlayParallelPayload, QueueActionPayload,
        QueueTrackPayload, BOT_ERROR, CHANGE_VOLUME, JOIN_VOICE_CHANNEL, LEAVE_VOICE_CHANNEL,
        LOOP_TRACK, MUTE_UNMUTE, PAUSE_PLAYBACK, PLAY_PARALLEL, QUEUE_TRACK, RESUME_PLAYBACK,
        SEEK_TRACK, SKIP_TRACK, TRACK_ENDED, TRACK_LOOPED, TRACK_PAUSED, TRACK_PLAYABLE,
        TRACK_PLAYED, UPDATED_GUILDS, UPDATE_PLAYER,
    },
    files::Track as TrackData,
    parallel::{Parallel, ParallelTracks},
    stores::{BOT_TOKEN_SETTING, DISCORD_FILENAME, GUILDS_SETTING, SETTINGS_FILENAME},
    Error,
};

/// Wrapper struct to check if a serenity client already exists
pub struct IsSerenityClientOn(pub bool);

#[derive(Serialize, Deserialize, Debug, Eq, Hash)]
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

struct ParallelKey;

impl TypeMapKey for ParallelKey {
    type Value = ParallelTracks;
}

pub struct Handler {
    app: AppHandle,
}

#[async_trait]
impl EventHandler for Handler {
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

        // Each callback needs to have ownership of whatever it uses since it outlives this functions
        // The need for a tokio::spawn to permit await calls causes annoying double-cloning of Arcs
        // because they need to be moved twice (first in the callback, then in the tokio async task)
        // Performance isn't a concern for these callbacks, but it's just kind of ugly
        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(JOIN_VOICE_CHANNEL, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                join_voice_channel(ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(LEAVE_VOICE_CHANNEL, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                leave_voice_channel(ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(QUEUE_TRACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_track(ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        let ctx1 = Arc::new(ctx);
        self.app.listen(PLAY_PARALLEL, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            let ctx = Arc::clone(&ctx1);
            tokio::spawn(async move {
                play_parallel(ev, &manager, &app, &ctx).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(RESUME_PLAYBACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Resume, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(PAUSE_PLAYBACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Pause, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(SKIP_TRACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Skip, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(LOOP_TRACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Loop, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(SEEK_TRACK, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Seek, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(CHANGE_VOLUME, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::ChangeVolume, ev, &manager, &app).await;
            });
        });

        let (manager1, app1) = clone_boilerplate(&manager, &self.app);
        self.app.listen(MUTE_UNMUTE, move |ev| {
            let (manager, app) = clone_boilerplate(&manager1, &app1);
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
    let mut client_exists = client_exists_mutex.lock().await;
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
        .register_songbird()
        .await?;

    tokio::spawn(async move {
        if let Err(why) = ds_client.start().await {
            println!("Client error: {why:?}");
        }
    });

    client_exists.0 = true;

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
    if let Err(err) = manager.join(payload.guildId, payload.channelId).await {
        print_emit_error(BOT_ERROR, &format!("Failed to join channel. {err}"), &app);
    };
}

async fn leave_voice_channel(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle) {
    let payload: GuildIdPayload = serde_json::from_str(ev.payload()).unwrap();
    let bot_is_in_a_call = manager.get(payload.guildId).is_some();
    if bot_is_in_a_call {
        manager
            .remove(payload.guildId)
            .await
            .expect("Failed to leave channel");
        app.emit(UPDATE_PLAYER, json!({ "playing": false }))
            .unwrap();
    } else {
        print_emit_error(BOT_ERROR, "Bot not in a voice channel", &app);
    };
}

async fn queue_track(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle) {
    let payload: QueueTrackPayload = serde_json::from_str(ev.payload()).unwrap();
    if let Some(handler_lock) = manager.get(payload.guildId) {
        let mut handler = handler_lock.lock().await;
        let mut track: Track = songbird::input::File::new(payload.trackData.path.clone()).into();
        track.volume = payload.volume;
        if payload.looping {
            track = track.loops(LoopState::Infinite);
        }
        add_trackevent_relays(&app, &mut track, &payload.trackData.path, false);

        let response = if payload.overwrite {
            // Unfortunately, the Queued type has private fields so I can't
            // initialize the new track manually despite having the TrackHandle. This means
            // I have to first put the track in the queue with the builtin method
            // and then move it after
            if handler.queue().is_empty() {
                handler.enqueue(track).await;
            } else {
                handler.enqueue(track).await;
                handler.queue().modify_queue(|q| {
                    let curr = q.pop_front().unwrap();
                    curr.stop().unwrap();
                    let new = q.pop_back().unwrap();
                    new.play().unwrap();
                    q.push_front(new);
                });
            }
            json!({ "position": 0 })
        } else if payload.prepend {
            let _ = handler.enqueue(track).await;
            handler.queue().modify_queue(|q| {
                // Pause and seek to zero the current track
                let curr_track = q.front().expect("Guaranteed to exist");
                curr_track.pause().expect("Failed to pause current track");
                curr_track
                    .seek(Duration::ZERO)
                    .result()
                    .expect("Failed to set seek current track to zero");
                // Move the new track from then end of the queue to the start
                // Popping it off the queue seems to pause it, so make sure to play it
                let new = q.pop_back().expect("Guaranteed to exist");
                new.play().expect("Failed to play new track");
                q.push_front(new);
            });
            json!({ "position": 0 })
        } else {
            let _ = handler.enqueue(track).await;
            json!({})
        };
        app.emit(UPDATE_PLAYER, response).unwrap();
    } else {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
    };
}

async fn play_parallel(ev: tauri::Event, manager: &Arc<Songbird>, app: &AppHandle, ctx: &Context) {
    let payload: PlayParallelPayload = serde_json::from_str(ev.payload()).unwrap();
    if let Some(handler_lock) = manager.get(payload.guildId) {
        let mut handler = handler_lock.lock().await;
        let parallel_tracks = {
            let typemap = ctx.data.read().await;
            typemap
                .get::<ParallelKey>()
                .expect("Guaranteed to exist")
                .clone()
        };

        let mut track: Track = songbird::input::File::new(payload.trackData.path.clone()).into();
        track.volume = payload.volume;
        if payload.looping {
            track = track.loops(LoopState::Infinite);
        }
        add_trackevent_relays(&app, &mut track, &payload.trackData.path, true);

        handler.add_parallel_track(track, &parallel_tracks);
        // handler.play(track);
    } else {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
    };
}

enum QueueAction {
    Resume,
    Pause,
    Skip,
    Stop,
    Loop,
    Seek,
    ChangeVolume,
}

async fn queue_action(
    action: QueueAction,
    ev: tauri::Event,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) {
    let payload: QueueActionPayload = serde_json::from_str(ev.payload()).unwrap();
    let maybe_handler = manager.get(payload.guildId);
    if let None = maybe_handler {
        print_emit_error(BOT_ERROR, "Not in a voice channel", &app);
        return;
    };

    let lock = maybe_handler.unwrap();
    let handler = lock.lock().await;
    let queue = handler.queue();

    let msg_str = match action {
        QueueAction::Resume => "resume",
        QueueAction::Pause => "pause",
        QueueAction::Skip => "skip",
        QueueAction::Stop => "stop",
        QueueAction::Loop => "activate loop for",
        QueueAction::Seek => "seek",
        QueueAction::ChangeVolume => "change volume",
    };
    // Most of these are handled automatically by the TrackEvent relays
    let response_event = match action {
        QueueAction::Resume => None,
        QueueAction::Pause => None,
        QueueAction::Skip => None,
        QueueAction::Stop => None,
        QueueAction::Loop => None,
        QueueAction::Seek => Some(UPDATE_PLAYER),
        QueueAction::ChangeVolume => None,
    };

    let result = match action {
        QueueAction::Resume => Some((queue.resume(), json!({}))),
        QueueAction::Pause => Some((queue.pause(), json!({}))),
        QueueAction::Skip => Some((queue.skip(), json!({}))),
        QueueAction::Stop => {
            queue.stop();
            None
        }
        QueueAction::Loop => {
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
                Some((res, json!({})))
            } else {
                None
            }
        }
        QueueAction::Seek => {
            if let Some(curr_track) = queue.current() {
                if let Some(position) = payload.position {
                    let time = Duration::from_secs(position);
                    let res = curr_track.seek_async(time).await;
                    // Drop the duration inside to avoid a type mismatch with other arms
                    let res = res.map(|_| ());
                    Some((res, json!({ "position": position })))
                } else {
                    None
                }
            } else {
                None
            }
        }
        QueueAction::ChangeVolume => {
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
            None
        }
    };

    match result {
        Some((Err(err), _)) => {
            print_emit_error(
                BOT_ERROR,
                &format!("Failed to {msg_str} track. {err}"),
                &app,
            );
        }
        Some((Ok(_), payload)) => {
            if let Some(event) = response_event {
                app.emit(event, payload).unwrap();
            }
        }
        None => (),
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
    eprintln!("{error_msg}");
    app.emit(ev_name, format!("{error_msg}")).unwrap();
}

/// Clone some references to have them moved into callbacks/tasks.
fn clone_boilerplate(manager: &Arc<Songbird>, app: &AppHandle) -> (Arc<Songbird>, AppHandle) {
    let manager1 = Arc::clone(manager);
    let app1 = app.clone();
    return (manager1, app1);
}

/// Add event handlers to a `Track` so that `TrackEvents` also fire Tauri events.
/// This allows the rest of the Tauri app to "see" the `TrackEvents` in real time.
/// These are not added as global events because only main queue tracks should fire
/// these. Parallel tracks behave differently.
fn add_trackevent_relays(app: &AppHandle, track: &mut Track, path: &PathBuf, is_parallel: bool) {
    track.events.add_event(
        EventData::new(
            songbird::Event::Track(TrackEvent::End),
            RelayTrackEnd {
                app: app.clone(),
                is_parallel,
                path: path.to_path_buf(),
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
                path: path.to_path_buf(),
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
                path: path.to_path_buf(),
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
                path: path.to_path_buf(),
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
                path: path.to_path_buf(),
            },
        ),
        Duration::ZERO,
    );
}

/* EVENT HANDLERS */
struct RelayTrackEnd {
    app: AppHandle,
    is_parallel: bool,
    path: PathBuf,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackEnd {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_ENDED,
                json!({ "is_parallel": self.is_parallel, "path": self.path }),
            )
            .expect(&format!("Couldn't emit {TRACK_ENDED} event"));
        return None;
    }
}

struct RelayTrackLoop {
    app: AppHandle,
    is_parallel: bool,
    path: PathBuf,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackLoop {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_LOOPED,
                json!({ "is_parallel": self.is_parallel, "path": self.path }),
            )
            .expect(&format!("Couldn't emit {TRACK_LOOPED} event"));
        return None;
    }
}

struct RelayTrackPlayable {
    app: AppHandle,
    is_parallel: bool,
    path: PathBuf,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPlayable {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PLAYABLE,
                json!({ "is_parallel": self.is_parallel, "path": self.path }),
            )
            .expect(&format!("Couldn't emit {TRACK_PLAYABLE} event"));
        return None;
    }
}

struct RelayTrackPlay {
    app: AppHandle,
    is_parallel: bool,
    path: PathBuf,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPlay {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PLAYED,
                json!({ "is_parallel": self.is_parallel, "path": self.path }),
            )
            .expect(&format!("Couldn't emit {TRACK_PLAYED} event"));
        return None;
    }
}

struct RelayTrackPause {
    app: AppHandle,
    is_parallel: bool,
    path: PathBuf,
}

#[async_trait]
impl VoiceEventHandler for RelayTrackPause {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(
                TRACK_PAUSED,
                json!({ "is_parallel": self.is_parallel, "path": self.path }),
            )
            .expect(&format!("Couldn't emit {TRACK_PAUSED} event"));
        return None;
    }
}
