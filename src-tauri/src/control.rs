#![allow(deprecated)] // Remove this one we switch to Poise

use serenity::{
    all::standard::{macros::command, CommandResult},
    prelude::*,
};
use songbird::{
    id::{ChannelId, GuildId},
    SerenityInit,
};
use std::sync::Arc;

#[command]
pub async fn join(ctx: &Context) -> CommandResult {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client not found.");

    let handler_lock = manager
        .join(guild_id, channel_id)
        .await
        .map_err(|e| e.to_string())?;

    // let mut handler = handler_lock.lock().await;

    // Optionally, you can start playing something here
    // handler.play_source(input).await?;

    Ok(())
}

#[command]
pub async fn leave(ctx: &Context, guild_id: GuildId) -> Result<(), String> {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client not found.");

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.stop();
    }

    manager.remove(guild_id).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn stop(ctx: &Context, guild_id: GuildId) -> Result<(), String> {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client not found.");

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.stop();
    }

    Ok(())
}

// #[command]
// pub async fn resume(ctx: &Context, guild_id: GuildId) -> Result<(), String> {
//     let manager = songbird::get(&ctx)
//         .await
//         .expect("Songbird Voice client not found.");

//     if let Some(handler_lock) = manager.get(guild_id) {
//         let mut handler = handler_lock.lock().await;
//         handler.queue().resume();
//     }

//     Ok(())
// }
