use std::{
    collections::HashSet,
    fs::{self, DirEntry, File},
    path::PathBuf,
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, StandardTagKey},
    probe::Hint,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::{
    settings::{AppSettings, AudioSource},
    Error,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub track_name: String,
    pub album: String,
    pub artist: String,
    pub path: PathBuf,
    pub extension: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TrackList {
    pub tracks: HashSet<Track>,
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
    let lock = settings.lock().map_err(|_| {
        return Error::Poison(
            "Could not lock AppSettings mutex. Can't return AudioSources".to_string(),
        );
    })?;

    return Ok(lock.audio_sources.clone());
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

#[tauri::command]
pub async fn refresh_audio_files(
    settings: State<'_, Mutex<AppSettings>>,
    tracklist: State<'_, Mutex<TrackList>>,
) -> Result<HashSet<Track>, Error> {
    let audio_sources = &settings
        .lock()
        .map_err(|_| {
            return Error::Poison(
                "Could not lock AppSettings mutex. Can't update AudioSources".to_string(),
            );
        })?
        .audio_sources;

    let tracks = &mut tracklist
        .lock()
        .map_err(|_| {
            return Error::Poison(
                "Could not lock TrackList mutex. Can't access Tracks".to_string(),
            );
        })?
        .tracks;

    for source in audio_sources {
        if source.recursive {
            todo!()
        } else {
            for maybe_entry in source.path.read_dir()? {
                if let Ok(entry) = maybe_entry {
                    if let Some(track) = get_track_from_direntry(entry) {
                        tracks.insert(track);
                    }
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        println!("{:#?}", tracks);
    }

    return Ok(tracks.clone());
}

fn get_track_from_direntry(direntry: DirEntry) -> Option<Track> {
    let filetype = direntry.file_type().ok();
    if let None = filetype {
        return None;
    }

    let filename = direntry.file_name();
    let filename_str = filename.to_string_lossy();
    let file_ext = filename_str.split(".").last().unwrap_or("").to_lowercase();

    let exts = ["ogg", "mp3", "wav", "flac"];
    if !filetype.unwrap().is_file() || !exts.contains(&file_ext.as_str()) {
        return None;
    }

    let file_ext_with_dot = format!(".{file_ext}");

    let (album, artist, track_name) =
        get_audio_metadata(&direntry, &file_ext).unwrap_or_else(|_err| {
            let track_name = filename_str.to_string().replace(&file_ext_with_dot, "");
            return (
                track_name,
                "Unknown Album".to_string(),
                "Unknown Artist".to_string(),
            );
        });

    return Some(Track {
        track_name,
        album,
        artist,
        path: direntry.path(),
        extension: file_ext,
    });
}

fn get_audio_metadata(file: &DirEntry, file_ext: &str) -> Result<(String, String, String), Error> {
    let source = File::open(file.path())?;
    let mss = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(file_ext);

    let meta_opts = MetadataOptions::default();
    let format_opts = FormatOptions::default();
    let probed = symphonia::default::get_probe().format(&hint, mss, &format_opts, &meta_opts)?;
    let mut format = probed.format;
    let mut meta = format.metadata();
    let revision = meta.skip_to_latest();

    let mut track_name = "Unknown Track".to_string();
    let mut album = "Unknown Album".to_string();
    let mut artist = "Unknown Artist".to_string();

    if let Some(revision) = revision {
        let tags = revision.tags();

        for tag in tags {
            if let Some(stdkey) = tag.std_key {
                match stdkey {
                    StandardTagKey::Album => album = tag.value.to_string(),
                    StandardTagKey::Artist => artist = tag.value.to_string(),
                    StandardTagKey::Composer => artist = tag.value.to_string(),
                    StandardTagKey::TrackTitle => track_name = tag.value.to_string(),
                    _ => {}
                }
            }
        }
    }

    return Ok((album, artist, track_name));
}
