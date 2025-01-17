mod discord;
mod events;
mod files;
mod parallel;
mod queue;
mod stores;

use discord::IsSerenityClientOn;
use serde_json::json;
use stores::{DISCORD_FILENAME, GUILDS_SETTING};
use tauri::Manager;
use tauri_plugin_fs::FsExt;
use tauri_plugin_store::StoreExt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Store(#[from] tauri_plugin_store::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
    #[error("Poisoned mutex. {0}")]
    PoisonedMutex(String),
    #[error("{0}")]
    Source(String),
    #[error("Operation cancelled. {0}")]
    Cancelled(String),
    #[error(transparent)]
    TauriError(#[from] tauri::Error),
    #[error("Discord client already exists")]
    SerenityClientAlreadyExists(),
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
    #[error("The payload was malformed. {0}")]
    BadPayload(String),
    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(tokio::sync::Mutex::new(IsSerenityClientOn(false)))
        .invoke_handler(tauri::generate_handler![
            files::add_audio_sources,
            files::update_audio_source,
            files::delete_audio_source,
            files::update_tracks_from_sources,
            discord::create_discord_client,
            discord::is_bot_connected,
        ])
        .setup(|app| {
            // Reset Discord guilds on startup to avoid stale data
            let store = app.store(DISCORD_FILENAME)?;
            store.set(GUILDS_SETTING, json!([]));

            let scope = app.fs_scope();
            scope.allow_directory(app.path().app_cache_dir()?, true)?;

            return Ok(());
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| eprintln!("Error while running Tauri application. Error: {:?}", err));
}
