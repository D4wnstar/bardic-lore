use std::{
    collections::HashSet,
    fs::{DirEntry, File},
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, MetadataRevision, StandardTagKey},
    probe::Hint,
    units::Time,
};
use tauri::{AppHandle, Wry};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::{Store, StoreExt};

use crate::{
    stores::{AUDIO_SOURCES_SETTING, SETTINGS_FILENAME, TRACKS_FILENAME, TRACKS_SETTING},
    Error,
};

/* DATA STRUCTURES */
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<u64>,
    pub path: PathBuf,
    pub extension: String,
}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct AudioSource {
    pub path: PathBuf,
    pub recursive: bool,
    pub active: bool,
}

impl AudioSource {
    pub fn new(path: PathBuf) -> Self {
        return AudioSource {
            path,
            recursive: false,
            active: true,
        };
    }
}

impl PartialOrd for AudioSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AudioSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let maybe_name1 = self.path.file_stem();
        let maybe_name2 = other.path.file_stem();
        if let None = maybe_name1 {
            return std::cmp::Ordering::Less;
        }
        if let None = maybe_name2 {
            return std::cmp::Ordering::Greater;
        }
        return maybe_name1.unwrap().cmp(maybe_name2.unwrap());
    }
}

/* TAURI COMMANDS */
#[tauri::command]
pub async fn add_audio_sources(app: AppHandle) -> Result<HashSet<AudioSource>, Error> {
    let paths = app.dialog().file().blocking_pick_folders();

    let store = app.store(SETTINGS_FILENAME)?;

    if let Some(paths) = paths {
        let sources = paths
            .iter()
            .map(|path| AudioSource::new(path.clone().into_path().unwrap()));
        let mut audio_sources = get_sources_from_store(&store)?;
        audio_sources.extend(sources);
        set_sources_in_store(&audio_sources, &store)?;
        return Ok(audio_sources);
    } else {
        return Err(Error::Cancelled("No paths selected".to_string()));
    }
}

#[tauri::command]
pub async fn delete_audio_source(
    path: PathBuf,
    app: AppHandle,
) -> Result<HashSet<AudioSource>, Error> {
    let store = app.store(SETTINGS_FILENAME)?;
    let mut audio_sources = get_sources_from_store(&store)?;

    let source_to_remove: Option<AudioSource> =
        audio_sources.iter().find(|src| src.path == path).cloned();

    if let Some(source) = source_to_remove {
        audio_sources.remove(&source);
    }

    set_sources_in_store(&audio_sources, &store)?;

    return Ok(audio_sources.clone());
}

#[tauri::command]
pub async fn update_audio_source(
    app: AppHandle,
    old_path: PathBuf,
    path: PathBuf,
    active: bool,
    recursive: bool,
) -> Result<HashSet<AudioSource>, Error> {
    let updated_source = AudioSource {
        path,
        active,
        recursive,
    };

    let store = app.store(SETTINGS_FILENAME)?;
    let mut audio_sources = delete_audio_source(old_path, app.clone()).await?;
    audio_sources.insert(updated_source);
    set_sources_in_store(&audio_sources, &store)?;
    return Ok(audio_sources.clone());
}

#[tauri::command]
pub async fn refresh_audio_files(app: AppHandle) -> Result<Vec<Track>, Error> {
    let settings_store = app.store(SETTINGS_FILENAME)?;
    let audio_sources = get_sources_from_store(&settings_store)?;

    let mut tracks: Vec<Track> = vec![];

    for source in audio_sources {
        if source.recursive {
            todo!()
        } else if source.active {
            for maybe_entry in source.path.read_dir()? {
                if let Ok(entry) = maybe_entry {
                    if let Some(track) = get_track_from_direntry(entry) {
                        tracks.push(track);
                    }
                }
            }
        }
    }

    tracks.sort();

    let tracks_store = app.store(TRACKS_FILENAME)?;
    tracks_store.set(TRACKS_SETTING, serde_json::to_value(tracks.clone())?);
    return Ok(tracks.clone());
}

fn get_track_from_direntry(direntry: DirEntry) -> Option<Track> {
    let filetype = direntry.file_type().ok();
    if let None = filetype {
        return None;
    }
    if !filetype.unwrap().is_file() {
        return None;
    }

    let filename = direntry.file_name();
    let filename = filename.to_string_lossy();
    let path = direntry.path();
    let file_ext = path.extension().map(|s| s.to_str()).flatten().unwrap_or("");
    let file_ext_with_dot = format!(".{file_ext}");

    let (album, artist, track_name, duration) = get_audio_metadata(&direntry, &file_ext)
        .unwrap_or_else(|_err| {
            let track_name = filename.to_string().replace(&file_ext_with_dot, "");
            return (Some(track_name), None, None, None);
        });

    return Some(Track {
        title: track_name.unwrap_or(filename.to_string().replace(&file_ext_with_dot, "")),
        album,
        artist,
        duration: duration.map(|t| t.seconds),
        path: direntry.path(),
        extension: file_ext.to_string(),
    });
}

fn get_audio_metadata(
    file: &DirEntry,
    file_ext: &str,
) -> Result<(Option<String>, Option<String>, Option<String>, Option<Time>), Error> {
    let source = File::open(file.path())?;
    let mss = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(file_ext);

    let meta_opts = MetadataOptions::default();
    let format_opts = FormatOptions::default();
    let mut probed =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &meta_opts)?;

    let duration = {
        let track = &probed.format.tracks().first();
        if let Some(track) = track {
            let params = &track.codec_params;
            let n_frames = params.n_frames;
            let time_base = params.time_base;
            if let Some(n_frames) = n_frames {
                if let Some(time_base) = time_base {
                    Some(time_base.calc_time(n_frames))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    let mut track_name = None;
    let mut album = None;
    let mut artist = None;

    let mut get_tags = |metadata: &MetadataRevision| {
        for tag in metadata.tags() {
            if let Some(key) = tag.std_key {
                match key {
                    StandardTagKey::Album => album = Some(tag.value.to_string()),
                    StandardTagKey::Artist => artist = Some(tag.value.to_string()),
                    StandardTagKey::Composer => artist = Some(tag.value.to_string()),
                    StandardTagKey::TrackTitle => track_name = Some(tag.value.to_string()),
                    _ => (),
                }
            }
        }
    };

    if let Some(metadata_rev) = probed.format.metadata().current() {
        get_tags(metadata_rev);
    } else if let Some(metadata_rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        get_tags(metadata_rev);
    }

    return Ok((album, artist, track_name, duration));
}

/* CONVENIENCE FUNCTIONS */
/// Type safe getter for audio sources. Will return an empty HashSet if not found in store.
fn get_sources_from_store(store: &Arc<Store<Wry>>) -> Result<HashSet<AudioSource>, Error> {
    let value = store.get(AUDIO_SOURCES_SETTING).unwrap_or("[]".into());
    let sources = serde_json::from_value(value)?;
    return Ok(sources);
}

/// Type safe setter for audio sources. Sorts sources alphabetically before saving.
fn set_sources_in_store(
    audio_sources: &HashSet<AudioSource>,
    store: &Arc<Store<Wry>>,
) -> Result<(), Error> {
    let mut vec: Vec<&AudioSource> = audio_sources.iter().collect();
    vec.sort();
    store.set(AUDIO_SOURCES_SETTING, serde_json::to_value(vec)?);

    return Ok(());
}
