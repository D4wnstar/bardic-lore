use std::{collections::HashSet, fs, path::PathBuf, sync::Mutex};

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::{
    settings::{AppSettings, AudioSource},
    Error,
};

#[tauri::command]
pub async fn add_audio_sources(
    app: AppHandle,
    settings: State<'_, Mutex<AppSettings>>,
) -> Result<HashSet<AudioSource>, Error> {
    println!("Opening dir selection");
    let paths = app.dialog().file().blocking_pick_folders();

    if let Some(paths) = paths {
        let sources = paths
            .iter()
            .map(|path| AudioSource::from_path(path.to_owned()));
        let mut lock = settings.lock().map_err(|_| {
            return Error::Poison(
                "Could not lock AppSettings mutex. Dropping selection".to_string(),
            );
        })?;
        lock.audio_sources.extend(sources);

        #[cfg(debug_assertions)]
        {
            println!("Audio sources: {:?}", lock.audio_sources);
        }

        return Ok(lock.audio_sources.clone());
    } else {
        return Err(Error::Cancelled("No paths selected".to_string()));
    }
}

#[tauri::command]
pub fn get_audio_sources(
    settings: State<'_, Mutex<AppSettings>>,
) -> Result<HashSet<AudioSource>, Error> {
    match settings.lock() {
        Ok(lock) => return Ok(lock.audio_sources.clone()),
        Err(_err) => {
            return Err(Error::Poison(
                "Could not lock AppSettings mutex. Can't return AudioSources".to_string(),
            ))
        }
    }
}

fn delete_audio_source_internal(path: PathBuf, settings: &mut AppSettings) {
    let source_to_remove: Option<AudioSource>;
    if let Some(old_source) = settings.audio_sources.iter().find(|s| s.path == path) {
        source_to_remove = Some(old_source.clone());
    } else {
        source_to_remove = None;
    }

    if let Some(source) = source_to_remove {
        settings.audio_sources.remove(&source);
    }
}

#[tauri::command]
pub async fn delete_audio_source(
    path: PathBuf,
    settings: State<'_, Mutex<AppSettings>>,
) -> Result<HashSet<AudioSource>, Error> {
    let mut lock = settings.lock().map_err(|_| {
        return Error::Poison(
            "Could not lock AppSettings mutex. Can't update AudioSources".to_string(),
        );
    })?;

    delete_audio_source_internal(path, &mut lock);

    #[cfg(debug_assertions)]
    {
        println!("{:?}", lock.audio_sources);
    }

    return Ok(lock.audio_sources.clone());
}

#[tauri::command]
pub async fn update_audio_source(
    old_path: PathBuf,
    path: PathBuf,
    active: bool,
    recursive: bool,
    settings: State<'_, Mutex<AppSettings>>,
) -> Result<HashSet<AudioSource>, Error> {
    let mut lock = settings.lock().map_err(|_| {
        return Error::Poison(
            "Could not lock AppSettings mutex. Can't update AudioSources".to_string(),
        );
    })?;

    delete_audio_source_internal(old_path, &mut lock);

    let updated_source = AudioSource {
        path,
        active,
        recursive,
    };
    lock.audio_sources.replace(updated_source);

    #[cfg(debug_assertions)]
    {
        println!("{:?}", lock.audio_sources);
    }

    return Ok(lock.audio_sources.clone());
}

pub fn save_settings_to_disk(
    settings: State<Mutex<AppSettings>>,
    config_path: &PathBuf,
) -> Result<(), Error> {
    match settings.lock() {
        Ok(stlock) => {
            let out_settings = toml::to_string_pretty(&*stlock)?;
            fs::write(config_path.clone(), out_settings)?;
        }
        Err(_err) => {
            return Err(Error::Poison(
                "Could not lock AppSettings mutex. Can't save to disk".to_string(),
            ))
        }
    };

    return Ok(());
}
