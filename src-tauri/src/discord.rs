use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    all::{
        ChannelId, ChannelType, Context, EventHandler, GatewayIntents, Guild, GuildId, Message,
        Ready,
    },
    async_trait,
    prelude::TypeMapKey,
};

// An HTTP client for yt-dlp to operate
use reqwest::Client as HttpClient;
use songbird::SerenityInit;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::{DISCORD_FILENAME, GUILDS_NAME};

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct TauriApp;

impl TypeMapKey for TauriApp {
    type Value = AppHandle;
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
}

impl PartialEq for VoiceChannelSlug {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct Handler;

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

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild_ids: Vec<GuildId> = ready.guilds.iter().map(|g| g.id).collect();

        // let valid_guild_names: Vec<String> = valid_guild_ids
        //     .iter()
        //     .filter_map(|id| ctx.cache.guild(id))
        //     .map(|g| g.name.clone())
        //     .collect();

        // let mut valid_channels = vec![];
        // for id in valid_guild_ids.iter() {
        //     if let Ok(channel) = id.channels(&ctx.http).await {
        //         valid_channels.push(channel);
        //     }
        // }
        // let channel_names_to_ids: Vec<HashMap<String, ChannelId>> = valid_channels
        //     .iter()
        //     .map(|map| {
        //         map.iter().fold(HashMap::new(), |mut acc, kv_pair| {
        //             if kv_pair.1.kind == ChannelType::Voice {
        //                 acc.insert(kv_pair.1.name.clone(), kv_pair.0.clone());
        //             }

        //             return acc;
        //         })
        //     })
        //     .collect();

        println!("GUILD IDS: {:#?}", guild_ids);
        // println!("GUILDS: {:#?}", valid_guild_names);
        // println!("CHANNELS: {:#?}", channel_names_to_ids);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        println!("GUILD: {}", guild.name);
        let voice_channels: Vec<VoiceChannelSlug> = guild
            .channels
            .iter()
            .filter_map(|(id, channel)| {
                if channel.kind == ChannelType::Voice {
                    return Some(VoiceChannelSlug {
                        id: id.clone(),
                        name: channel.name.clone(),
                    });
                } else {
                    return None;
                }
            })
            .collect();

        if let Some(app_handle) = ctx.data.read().await.get::<TauriApp>() {
            if let Ok(store) = app_handle.store(DISCORD_FILENAME) {
                let guilds_value = store.get(GUILDS_NAME).unwrap_or(json!([]));
                let mut guilds: HashSet<GuildSlug> =
                    serde_json::from_value(guilds_value).unwrap_or(HashSet::new());
                guilds.insert(GuildSlug {
                    id: guild.id,
                    name: guild.name,
                    voice_channels,
                });
                if let Ok(value) = serde_json::to_value(guilds) {
                    store.set(GUILDS_NAME, value);
                }
            }
        }
    }

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
pub async fn create_discord_client(app: AppHandle) {
    // Get token from store
    let token = "token here";
    let mut ds_client = serenity::Client::builder(
        token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler)
    .register_songbird()
    .type_map_insert::<HttpKey>(reqwest::Client::new())
    .type_map_insert::<TauriApp>(app.clone())
    .await
    .expect("Error creating Discord client");

    tokio::spawn(async move {
        if let Err(why) = ds_client.start().await {
            println!("Client error: {why:?}");
        }
    });
}
