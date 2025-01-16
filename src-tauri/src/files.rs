use std::{
    collections::HashSet,
    fs::{File, FileType},
    hash::Hash,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, MetadataRevision, StandardTagKey},
    probe::Hint,
    units::Time,
};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;
use walkdir::WalkDir;

use crate::{
    stores::{AUDIO_SOURCES_SETTING, SETTINGS_FILENAME, TRACKS_FILENAME, TRACKS_SETTING},
    Error,
};

/* DATA STRUCTURES */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<u64>,
    pub path: PathBuf,
    pub extension: String,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Track {}

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

// Hashing for a Track should be delegated to its path
impl Hash for Track {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// AudioSources are equivalent based on their path.
impl PartialEq for AudioSource {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

// Hashing for an AudioSource should be delegated to its path
impl Hash for AudioSource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl Eq for AudioSource {}

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
/// Add audio sources with a folder picker dialog.
#[tauri::command]
pub async fn add_audio_sources(app: AppHandle) -> Result<HashSet<AudioSource>, Error> {
    let paths = app.dialog().file().blocking_pick_folders();

    if let Some(paths) = paths {
        let sources = paths
            .iter()
            .map(|path| AudioSource::new(path.clone().into_path().unwrap()));
        let mut audio_sources = get_sources_from_store(&app)?;
        audio_sources.extend(sources);
        set_sources_in_store(&app, &audio_sources)?;
        return Ok(audio_sources);
    } else {
        return Err(Error::Cancelled("No paths selected".to_string()));
    }
}

/// Delete the given audio source if it exists, returning whether it did.
#[tauri::command]
pub async fn delete_audio_source(app: AppHandle, source: AudioSource) -> Result<bool, Error> {
    let mut audio_sources = get_sources_from_store(&app)?;
    // Path equality is taken care of by PartialEq and Hash
    let removed = audio_sources.remove(&source);
    if removed {
        set_sources_in_store(&app, &audio_sources)?;
    }

    return Ok(removed);
}

/// Overwrites the audio source with the same path as the given one. Returns whether
/// something was updated or not.
#[tauri::command]
pub async fn update_audio_source(app: AppHandle, source: AudioSource) -> Result<bool, Error> {
    let mut audio_sources = get_sources_from_store(&app)?;
    // Overwriting by equal path is taken care of by PartialEq and Hash
    let updated = audio_sources.replace(source.clone());
    if updated.is_some() {
        set_sources_in_store(&app, &audio_sources)?;
    }

    return Ok(updated.is_some());
}

/// Update the track cache with the given sources. Active sources add track,
/// inactive sources remove them.
#[tauri::command]
pub async fn update_tracks_from_sources(
    app: AppHandle,
    sources: Option<HashSet<AudioSource>>,
) -> Result<(HashSet<Track>, HashSet<Track>), Error> {
    // Tracks from active sources get added, inactive ones get removed
    let mut tracks_to_add: HashSet<Track> = HashSet::new();
    let mut tracks_to_remove: HashSet<Track> = HashSet::new();
    let sources = sources.unwrap_or_else(|| get_sources_from_store(&app).unwrap_or_default());

    for source in sources {
        if source.recursive {
            for entry in WalkDir::new(source.path).into_iter().filter_map(|e| e.ok()) {
                if let Some(track) = make_track(
                    Some(entry.file_type()),
                    entry.file_name().to_string_lossy().to_string(),
                    entry.path().to_path_buf(),
                ) {
                    if source.active {
                        tracks_to_add.insert(track);
                    } else {
                        tracks_to_remove.insert(track);
                    }
                }
            }
        } else {
            for entry in source.path.read_dir()?.filter_map(|e| e.ok()) {
                if let Some(track) = make_track(
                    entry.file_type().ok(),
                    entry.file_name().to_string_lossy().to_string(),
                    entry.path(),
                ) {
                    if source.active {
                        tracks_to_add.insert(track);
                    } else {
                        tracks_to_remove.insert(track);
                    }
                }
            }
        }
    }

    add_tracks_to_store(&app, tracks_to_add.clone())?;
    remove_tracks_from_store(&app, tracks_to_remove.clone())?;

    return Ok((tracks_to_add.clone(), tracks_to_remove.clone()));
}

/// Attempt to transform the file at `path` into a `Track`.
fn make_track(filetype: Option<FileType>, filename: String, path: PathBuf) -> Option<Track> {
    if let None = filetype {
        return None;
    }
    if !filetype.unwrap().is_file() {
        return None;
    }

    let file_ext = path.extension().map(|s| s.to_str()).flatten().unwrap_or("");
    let file_ext_with_dot = format!(".{file_ext}");

    let (album, artist, track_name, duration) = get_audio_metadata(&path, &file_ext)
        .unwrap_or_else(|_err| {
            let track_name = filename.to_string().replace(&file_ext_with_dot, "");
            return (Some(track_name), None, None, None);
        });

    return Some(Track {
        title: track_name.unwrap_or(filename.to_string().replace(&file_ext_with_dot, "")),
        album,
        artist,
        duration: duration.map(|t| t.seconds),
        path: path.clone(),
        extension: file_ext.to_string(),
    });
}

/// Get some audio metadata from the file at `path`.
fn get_audio_metadata(
    path: &PathBuf,
    file_ext: &str,
) -> Result<(Option<String>, Option<String>, Option<String>, Option<Time>), Error> {
    let source = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(file_ext);

    let meta_opts = MetadataOptions::default();
    let format_opts = FormatOptions::default();
    let mut probed =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &meta_opts)?;

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

    // This assumes there is only one track per file. Some tracks have empty tracks with
    // invalid metadata data before. This finds the first valid track, if it exists
    let track = &probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.n_frames.is_some() && t.codec_params.time_base.is_some());
    let duration = {
        if let Some(track) = track {
            let params = &track.codec_params;
            let n_frames = params.n_frames.unwrap();
            let time_base = params.time_base.unwrap();
            Some(time_base.calc_time(n_frames))
        } else {
            None
        }
    };

    return Ok((album, artist, track_name, duration));
}

/* CONVENIENCE FUNCTIONS */
/// Type safe getter for audio sources. Will return an empty HashSet if not found in store.
fn get_sources_from_store(app: &AppHandle) -> Result<HashSet<AudioSource>, Error> {
    let store = app.store(SETTINGS_FILENAME)?;
    let value = store.get(AUDIO_SOURCES_SETTING).unwrap_or(json!([]));
    let sources = serde_json::from_value(value)?;
    return Ok(sources);
}

/// Type safe setter for audio sources. Sorts sources alphabetically before saving.
fn set_sources_in_store(
    app: &AppHandle,
    audio_sources: &HashSet<AudioSource>,
) -> Result<(), Error> {
    let store = app.store(SETTINGS_FILENAME)?;
    let mut vec: Vec<&AudioSource> = audio_sources.iter().collect();
    vec.sort();
    store.set(AUDIO_SOURCES_SETTING, serde_json::to_value(vec)?);
    store.save()?;

    return Ok(());
}

/// Type safe getter for tracks. Will return an empty HashSet if not found in store.
fn get_tracks_from_store(app: &AppHandle) -> Result<HashSet<Track>, Error> {
    let store = app.store(TRACKS_FILENAME)?;
    let value = store.get(TRACKS_SETTING).unwrap_or(json!([]));
    let tracks = serde_json::from_value(value)?;
    return Ok(tracks);
}

/// Type safe setter for tracks. This will add all the given tracks into the store,
/// ignoring duplicates.
fn add_tracks_to_store(app: &AppHandle, tracks: HashSet<Track>) -> Result<(), Error> {
    let store = app.store(TRACKS_FILENAME)?;
    let mut curr_tracks = get_tracks_from_store(app)?;
    curr_tracks.extend(tracks);
    store.set(TRACKS_SETTING, serde_json::to_value(curr_tracks)?);
    store.save()?;

    Ok(())
}

/// Type safe setter for tracks. This will remove all the given tracks from the store,
/// leaving all other tracks untouched.
fn remove_tracks_from_store(app: &AppHandle, tracks: HashSet<Track>) -> Result<(), Error> {
    let store = app.store(TRACKS_FILENAME)?;
    let mut curr_tracks = get_tracks_from_store(app)?;
    curr_tracks = curr_tracks.difference(&tracks).cloned().collect();
    store.set(TRACKS_SETTING, serde_json::to_value(curr_tracks)?);
    store.save()?;

    Ok(())
}
