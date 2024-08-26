mod files;
mod playback;
mod settings;

use std::{fs, path::PathBuf, sync::Mutex};

use files::{save_settings_to_disk, TrackList};
use playback::{GlobalStream, SinkList};
use settings::AppSettings;
use tauri::Manager;

const DATA_DIR_NAME: &str = "bardic_lore";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error("Poisoned mutex. {0}")]
    Poison(String),
    #[error("{0}")]
    Source(String),
    #[error("Operation cancelled. {0}")]
    Cancelled(String),
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

fn create_data_dir_if_not_exist() -> Result<PathBuf, Error> {
    let data_dir = dirs::data_dir().ok_or_else(|| {
        return std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Couldn't find data folder on this operating system",
        );
    })?;

    let bardic_lore_data_path = data_dir.join(DATA_DIR_NAME);
    if !bardic_lore_data_path.is_dir() {
        fs::create_dir(&bardic_lore_data_path)?;
    }

    return Ok(bardic_lore_data_path);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load the config file first
    let data_dir = create_data_dir_if_not_exist()
        .expect("Cannot create config directory. Do you have permission to change it?");
    let config_path = data_dir.join(CONFIG_FILE_NAME);

    let settings: AppSettings;
    let mut save_changes = true;
    if config_path.is_file() {
        let config_text = fs::read_to_string(&config_path).unwrap_or_default();
        settings = toml::from_str(&config_text).unwrap_or_else(|err| {
            eprintln!(
                "Invalid config file detected. Using default settings and will not save changes to disk. Please fix the config file manually ({}).\nError: {}",
                config_path.to_string_lossy(),
                err.to_string()
            );
            save_changes = false;
            return AppSettings::default();
        });
    } else {
        eprintln!("Config file not found. Using default settings");
        settings = AppSettings::default();
    }

    // Then start the app with those settings
    tauri::Builder::default()
        .manage(GlobalStream::default())
        .manage(SinkList::default())
        .manage(Mutex::new(settings))
        .manage(Mutex::new(TrackList::default()))
        .invoke_handler(tauri::generate_handler![
            playback::play,
            files::add_audio_sources,
            files::get_audio_sources,
            files::update_audio_source,
            files::delete_audio_source,
            files::refresh_audio_files,
        ])
        .on_window_event(move |window, event| match event {
            tauri::WindowEvent::Destroyed => {
                // And finally save settings to disk on app closure
                if !save_changes {
                    return;
                }
                let settings = window.app_handle().state::<Mutex<AppSettings>>();
                match save_settings_to_disk(settings, &config_path) {
                    Ok(_) => (),
                    Err(err) => eprintln!("{:?}", err.to_string()),
                }
            }
            _ => {}
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| eprintln!("Error while running tauri application. Error: {:?}", err));
}
