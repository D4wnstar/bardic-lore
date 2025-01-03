use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    all::{ChannelId, ChannelType, Context, EventHandler, GatewayIntents, Guild, GuildId, Message},
    async_trait,
    prelude::TypeMapKey,
};

use reqwest::Client as HttpClient;
use songbird::SerenityInit;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    settings::{BOT_TOKEN_SETTING, DISCORD_FILENAME, GUILDS_SETTING, SETTINGS_FILENAME},
    Error,
};

/// Wrapper struct to check if a serenity client already exists
pub struct IsSerenityClientOn(pub bool);

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

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
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Message: {}", msg.content);

        let http = ctx.http.clone();

        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&http, "Pong!").await {
                eprintln!("Error sending message: {why:?}");
            }
        } else if msg.content.starts_with("!join") {
            // let guild_id = match msg.guild_id {
            //     Some(id) => id,
            //     None => {
            //         if let Err(e) = msg
            //             .channel_id
            //             .say(&http, "This command can only be used in a server")
            //             .await
            //         {
            //             eprintln!("Error sending message: {e:?}");
            //         }
            //         return;
            //     }
            // };

            // let channel_id = match msg.content.split_whitespace().nth(1) {
            //     Some(id) => match id.parse::<u64>() {
            //         Ok(id) => id,
            //         Err(e) => {
            //             if let Err(e) = msg
            //                 .channel_id
            //                 .say(&http, format!("Invalid channel ID: {}", e))
            //                 .await
            //             {
            //                 eprintln!("Error sending message: {e:?}");
            //             }
            //             return;
            //         }
            //     },
            //     None => {
            //         if let Err(e) = msg
            //             .channel_id
            //             .say(&http, "Please specify a channel ID")
            //             .await
            //         {
            //             eprintln!("Error sending message: {e:?}");
            //         }
            //         return;
            //     }
            // };

            // let manager = songbird::get(&ctx).await.unwrap();

            // let _handler = manager.join(guild_id, channel_id).await;

            // if let Err(e) = msg.channel_id.say(&http, "Joined the voice channel").await {
            //     eprintln!("Error sending message: {e:?}");
            // }
        } else if msg.content == "!channels" {
            let guild_id = match msg.guild_id {
                Some(id) => id,
                None => {
                    if let Err(e) = msg
                        .channel_id
                        .say(&http, "This command can only be used in a server")
                        .await
                    {
                        eprintln!("Error sending message: {e:?}");
                    }
                    return;
                }
            };

            let guild = match ctx.cache.guild(guild_id) {
                Some(guild) => guild.clone(), // Clone the guild to ensure it is Send
                None => {
                    eprintln!("Failed to fetch guild information");
                    return;
                }
            };

            let mut channels_info = String::new();
            for channel in guild.channels.iter() {
                if channel.1.kind == ChannelType::Voice {
                    channels_info.push_str(&format!(
                        "Channel ID: {}, Name: {}\n",
                        channel.0, channel.1.name
                    ));
                }
            }

            if let Err(e) = msg.channel_id.say(&http, channels_info).await {
                eprintln!("Error sending message: {e:?}");
            }
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        // serenity has no API to tell the bot to do something from code,
        // it can only handle Gateway events sent by Discord, such as chat messages
        // We need to control the bot manually from the frontend though, so we
        // setup Tauri event listeners instead. Whenever we want the bot to do something on
        // command, we register a callback for an event right here and then emit that
        // event from anywhere

        let ctx = Arc::new(ctx);
        let manager = songbird::get(&ctx).await.unwrap();

        // Each callback needs to have ownership of whatever it needs since it outlives this functions
        let ctx1 = Arc::clone(&ctx);
        let manager1 = Arc::clone(&manager);
        self.app.listen("join-voice-channel", move |ev| {
            // The need for a tokio::spawn to allow for async causes annoying double-cloning of Arcs
            // because they need to be moved twice (first in the callback, then in the tokio async task)
            // Performance isn't a concern for these callbacks, but it's just kind of ugly
            let ctx = Arc::clone(&ctx1);
            let manager = Arc::clone(&manager1);
            tokio::spawn(async move {
                let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
                let maybe_guild_id = payload.get("guildId").and_then(|id| id.parse::<u64>().ok());
                let maybe_channel_id = payload
                    .get("channelId")
                    .and_then(|id| id.parse::<u64>().ok());
                if let Some(guild_id) = maybe_guild_id {
                    if let Some(channel_id) = maybe_channel_id {
                        // The frontend only holds the id number, we need the whole object
                        let (gid, cid) = {
                            let guild = ctx.cache.guild(guild_id).unwrap();
                            let channel =
                                guild.channels.iter().find(|c| *c.0 == channel_id).unwrap();
                            (guild.id, channel.0.clone())
                        };

                        manager
                            .join(gid, cid)
                            .await
                            .expect("Failed to join channel");
                    };
                }
            });
        });

        let ctx2 = Arc::clone(&ctx);
        let manager2 = Arc::clone(&manager);
        self.app.listen("leave-voice-channels", move |ev| {
            let ctx = Arc::clone(&ctx2);
            let manager = Arc::clone(&manager2);
            tokio::spawn(async move {
                let payload: HashMap<String, String> = serde_json::from_str(ev.payload()).unwrap();
                let maybe_guild_id = payload.get("guildId").and_then(|id| id.parse::<u64>().ok());
                if let Some(guild_id) = maybe_guild_id {
                    let gid = {
                        let guild = ctx.cache.guild(guild_id).unwrap();
                        guild.id
                    };
                    let has_handler = manager.get(gid).is_some();
                    if has_handler {
                        manager.remove(gid).await.expect("Failed to leave channel");
                    } else {
                        println!("Bot not in a voice channel")
                    }
                }
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
                self.app.emit("updated-guilds", ()).unwrap_or_else(|why| {
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

    // async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
    //     if let Some(old) = old {
    //         if old.guild_id != new.guild_id {
    //             return;
    //         }
    //     }

    //     let guild_id = match new.guild_id {
    //         Some(id) => id,
    //         None => return,
    //     };

    //     let channel_id = match new.channel_id {
    //         Some(id) => id,
    //         None => return,
    //     };

    //     let manager = songbird::get(&ctx).await.unwrap();

    //     if let Some(handler_lock) = manager.get(guild_id) {
    //         let mut handler = handler_lock.lock().await;
    //         if handler.queue().is_empty() {
    //             handler
    //                 .play_source(ffmpeg(channel_id.to_string()).await.unwrap())
    //                 .await
    //                 .unwrap();
    //         }
    //     }
    // }
}

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
    let mut ds_client = serenity::Client::builder(
        token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler { app: app.clone() })
    .register_songbird()
    // An HTTP client for yt-dlp to operate
    .type_map_insert::<HttpKey>(reqwest::Client::new())
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
