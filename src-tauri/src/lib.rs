mod files;
mod settings;

use std::collections::HashMap;
use std::sync::Mutex;

use files::TrackList;
use settings::AppSettings;
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
    Poison(String),
    #[error("{0}")]
    Source(String),
    #[error("Operation cancelled. {0}")]
    Cancelled(String),
    // #[error(transparent)]
    // Symphonia(#[from] symphonia::core::errors::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub const SETTINGS_FILENAME: &str = "settings.json";
pub const AUDIO_SOURCES_SETTING: &str = "audio-sources";

pub const TRACKS_FILENAME: &str = "tracks.json";
pub const TRACKS_NAME: &str = "tracks";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(TrackList::default()))
        .invoke_handler(tauri::generate_handler![
            files::add_audio_sources,
            files::get_audio_sources,
            files::update_audio_source,
            files::delete_audio_source,
            files::refresh_audio_files,
        ])
        .setup(|app| {
            let settings = AppSettings::default();
            let mut defaults = HashMap::new();
            defaults.insert(
                AUDIO_SOURCES_SETTING.into(),
                serde_json::to_value(settings.audio_sources)?,
            );

            app.store_builder(SETTINGS_FILENAME)
                .defaults(defaults)
                .build()?;

            let mut track_defaults = HashMap::new();
            track_defaults.insert(TRACKS_NAME.into(), serde_json::to_value("[]")?);
            app.store_builder(TRACKS_FILENAME)
                .defaults(track_defaults)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| eprintln!("Error while running tauri application. Error: {:?}", err));
}
