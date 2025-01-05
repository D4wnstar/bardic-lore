use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    all::{ChannelId, ChannelType, Context, EventHandler, GatewayIntents, Guild, GuildId},
    async_trait,
};

use reqwest::Client as HttpClient;
use songbird::{
    tracks::{LoopState, Track},
    EventContext, EventHandler as VoiceEventHandler, SerenityInit, Songbird, TrackEvent,
};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    events::{
        BOT_ERROR, JOIN_VOICE_CHANNEL, LEAVE_VOICE_CHANNEL, LOOP_TRACK, PAUSE_PLAYBACK,
        QUEUE_TRACK, RESUME_PLAYBACK, SKIP_TRACK, TRACK_ENDED, UPDATED_GUILDS, UPDATE_TRACK,
    },
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

pub struct Handler {
    app: AppHandle,
    http_client: HttpClient,
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

        let ctx = Arc::new(ctx);
        let manager = songbird::get(&ctx)
            .await
            .expect("Failed to get Songbird manager");

        // Each callback needs to have ownership of whatever it uses since it outlives this functions
        // The need for a tokio::spawn to permit await calls causes annoying double-cloning of Arcs
        // because they need to be moved twice (first in the callback, then in the tokio async task)
        // Performance isn't a concern for these callbacks, but it's just kind of ugly
        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(JOIN_VOICE_CHANNEL, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                join_voice_channel(ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(LEAVE_VOICE_CHANNEL, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                leave_voice_channel(ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(QUEUE_TRACK, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                queue_track(ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(RESUME_PLAYBACK, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Resume, ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(PAUSE_PLAYBACK, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Pause, ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(SKIP_TRACK, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Skip, ev, &ctx, &manager, &app).await;
            });
        });

        let (ctx1, manager1, app1) = clone_boilerplate(&ctx, &manager, &self.app);
        self.app.listen(LOOP_TRACK, move |ev| {
            let (ctx, manager, app) = clone_boilerplate(&ctx1, &manager1, &app1);
            tokio::spawn(async move {
                queue_action(QueueAction::Loop, ev, &ctx, &manager, &app).await;
            });
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
        .event_handler(Handler {
            app: app.clone(),
            http_client: reqwest::Client::new(),
        })
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

/* EVENT HANDLERS */
struct NotifyTrackEnd {
    app: AppHandle,
}

#[async_trait]
impl VoiceEventHandler for NotifyTrackEnd {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.app
            .emit(TRACK_ENDED, ())
            .expect("Couldn't emit TRACK_ENDED event");
        return None;
    }
}

/* BOT EVENT CALLBACKS */
async fn join_voice_channel(
    ev: tauri::Event,
    ctx: &Arc<Context>,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) {
    let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
    let guild_id = get_guild_id(&payload, &ctx, &app);
    if let None = guild_id {
        print_emit_error(BOT_ERROR, "Guild ID not in payload", &app);
        return;
    }
    let guild_id = guild_id.unwrap();

    let channel_id = get_channel_id(&guild_id, &payload, &ctx, &app);
    if let None = channel_id {
        print_emit_error(BOT_ERROR, "Channel ID not in payload", &app);
        return;
    }
    let channel_id = channel_id.unwrap();

    if let Err(err) = manager.join(guild_id, channel_id).await {
        print_emit_error(BOT_ERROR, &format!("Failed to join channel. {err}"), &app);
    } else {
        let handler_lock = manager.get(guild_id).unwrap();
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(
            songbird::Event::Track(TrackEvent::End),
            NotifyTrackEnd { app: app.clone() },
        );
    };
}

async fn leave_voice_channel(
    ev: tauri::Event,
    ctx: &Arc<Context>,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) {
    let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
    let guild_id = get_guild_id(&payload, &ctx, &app);
    if let None = guild_id {
        print_emit_error(BOT_ERROR, "Guild ID not in payload", &app);
        return;
    }
    let guild_id = guild_id.unwrap();
    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        manager
            .remove(guild_id)
            .await
            .expect("Failed to leave channel");
        app.emit(UPDATE_TRACK, json!({ "playing": false })).unwrap();
    } else {
        print_emit_error(BOT_ERROR, "Bot not in a voice channel", &app);
    };
}

async fn queue_track(
    ev: tauri::Event,
    ctx: &Arc<Context>,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) {
    let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
    let guild_id = get_guild_id(&payload, &ctx, &app);
    let filepath = payload.get("filepath").cloned();
    let looping = payload.get("loop");
    if let None = guild_id {
        print_emit_error(BOT_ERROR, "Guild ID not in payload", &app);
        return;
    }
    if let None = filepath {
        print_emit_error(BOT_ERROR, "Filepath not in payload", &app);
        return;
    }
    if let None = looping {
        print_emit_error(BOT_ERROR, "Looping not in payload. Not looping", &app);
    }

    if let Some(handler_lock) = manager.get(guild_id.unwrap()) {
        let mut handler = handler_lock.lock().await;
        let track: Track = songbird::input::File::new(filepath.unwrap()).into();
        let handle = handler.enqueue(track).await;
        if looping.unwrap().parse::<bool>().unwrap() {
            handle
                .enable_loop()
                .expect("Couldn't enable loop on new track");
        }
        app.emit(UPDATE_TRACK, json!({ "playing": true })).unwrap();
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
}

async fn queue_action(
    action: QueueAction,
    ev: tauri::Event,
    ctx: &Arc<Context>,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) {
    let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
    let guild_id = get_guild_id(&payload, &ctx, &app);
    if let None = guild_id {
        print_emit_error(BOT_ERROR, "Guild ID not in payload", &app);
        return;
    }

    let maybe_handler = manager.get(guild_id.unwrap());
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
    };
    let response_event = match action {
        QueueAction::Resume => Some(UPDATE_TRACK),
        QueueAction::Pause => Some(UPDATE_TRACK),
        QueueAction::Skip => None,
        QueueAction::Stop => todo!(),
        QueueAction::Loop => Some(UPDATE_TRACK),
    };

    let result = match action {
        QueueAction::Resume => Some((queue.resume(), json!({ "playing": true }))),
        QueueAction::Pause => Some((queue.pause(), json!({ "playing": false }))),
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
                let new_state;
                let res = match maybe_info.unwrap().loops {
                    LoopState::Infinite => {
                        new_state = false;
                        track.disable_loop()
                    }
                    LoopState::Finite(_) => {
                        new_state = true;
                        track.enable_loop()
                    }
                };
                Some((res, json!({ "looping": new_state })))
            } else {
                None
            }
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

/* CONVENIENCE FUNCTIONS */
/// Convenience function to turn a u64 guild ID from a payload into a proper GuildId object.
fn get_guild_id(
    payload: &HashMap<String, String>,
    ctx: &Context,
    app: &AppHandle,
) -> Option<GuildId> {
    let maybe_guild_id = payload.get("guildId");
    if let None = maybe_guild_id {
        return None;
    }
    let guild_id = {
        let gid = maybe_guild_id
            .unwrap()
            .parse::<u64>()
            .or_else(|err| {
                print_emit_error(BOT_ERROR, "Failed to parse Guild ID to u64", app);
                return Err(err);
            })
            .ok();
        if let None = gid {
            return None;
        }

        let guild = ctx.cache.guild(gid.unwrap()).unwrap();
        guild.id
    };

    return Some(guild_id);
}

/// Convenience function to turn a u64 channel ID from a payload into a proper ChannelId object.
fn get_channel_id(
    guild_id: &GuildId,
    payload: &HashMap<String, String>,
    ctx: &Context,
    app: &AppHandle,
) -> Option<ChannelId> {
    let maybe_channel_id = payload.get("channelId");
    if let None = maybe_channel_id {
        return None;
    }
    let channel_id = {
        let cid = maybe_channel_id
            .unwrap()
            .parse::<u64>()
            .or_else(|err| {
                print_emit_error(BOT_ERROR, "Failed to parse Channel ID to u64", app);
                return Err(err);
            })
            .ok();
        if let None = cid {
            return None;
        }

        let guild = ctx.cache.guild(guild_id).unwrap();
        let channel = guild
            .channels
            .iter()
            .find(|c| *c.0 == cid.unwrap())
            .unwrap();
        channel.0.clone()
    };

    return Some(channel_id);
}

/// Print an error to stderr and emit a Tauri event with the same message as the payload.
fn print_emit_error(ev_name: &str, error_msg: &str, app: &AppHandle) {
    eprintln!("{error_msg}");
    app.emit(ev_name, format!("{error_msg}")).unwrap();
}

/// Clone some references to have them moved into callbacks/tasks.
fn clone_boilerplate(
    ctx: &Arc<Context>,
    manager: &Arc<Songbird>,
    app: &AppHandle,
) -> (Arc<Context>, Arc<Songbird>, AppHandle) {
    let ctx1 = Arc::clone(ctx);
    let manager1 = Arc::clone(manager);
    let app1 = app.clone();
    return (ctx1, manager1, app1);
}
