use std::{
    collections::HashSet,
    fs::{File, FileType},
    hash::Hash,
    io::Cursor,
    path::PathBuf,
};

use image::{imageops::FilterType, ImageFormat, ImageReader, Pixel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, MetadataRevision, StandardTagKey, StandardVisualKey, Visual},
    probe::Hint,
    units::Time,
};
use tauri::{ipc::Channel, AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;
use tracing::warn;
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
    pub cover_hash: Option<String>,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "track", rename_all = "camelCase")]
pub enum TrackPacket {
    Add(Track),
    Remove(Track),
    Refresh(),
}

/// Update the track cache with the given sources. Active sources add track,
/// inactive sources remove them.
#[tauri::command]
pub async fn update_tracks_from_sources(
    app: AppHandle,
    on_get_track: Channel<TrackPacket>,
    sources: Option<HashSet<AudioSource>>,
    reset: Option<bool>,
) -> Result<(), Error> {
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
                    &app,
                ) {
                    if source.active {
                        tracks_to_add.insert(track.clone());
                        drop(on_get_track.send(TrackPacket::Add(track)));
                    } else {
                        tracks_to_remove.insert(track.clone());
                        drop(on_get_track.send(TrackPacket::Remove(track)));
                    }
                }
            }
        } else {
            for entry in source.path.read_dir()?.filter_map(|e| e.ok()) {
                if let Some(track) = make_track(
                    entry.file_type().ok(),
                    entry.file_name().to_string_lossy().to_string(),
                    entry.path(),
                    &app,
                ) {
                    if source.active {
                        tracks_to_add.insert(track.clone());
                        drop(on_get_track.send(TrackPacket::Add(track)));
                    } else {
                        tracks_to_remove.insert(track.clone());
                        drop(on_get_track.send(TrackPacket::Remove(track)));
                    }
                }
            }
        }
    }

    if let Some(_) = reset {
        let store = app.store(TRACKS_FILENAME)?;
        store.set(TRACKS_SETTING, serde_json::to_value(tracks_to_add.clone())?);
        store.save()?;
        drop(on_get_track.send(TrackPacket::Refresh()));
    } else {
        add_tracks_to_store(&app, tracks_to_add.clone())?;
        remove_tracks_from_store(&app, tracks_to_remove.clone())?;
    }

    return Ok(());
}

/// Attempt to transform the file at `path` into a `Track`.
fn make_track(
    filetype: Option<FileType>,
    filename: String,
    path: PathBuf,
    app: &AppHandle,
) -> Option<Track> {
    if let None = filetype {
        return None;
    }
    if !filetype.unwrap().is_file() {
        return None;
    }

    let file_ext = path.extension().map(|s| s.to_str()).flatten().unwrap_or("");
    let file_ext_with_dot = format!(".{file_ext}");

    let metadata = get_audio_metadata(&path, &file_ext, app)
        .inspect_err(|err| warn!("Failed to get audio metadata for {path:?}. Error: {err}"))
        .unwrap_or(TrackMetadata {
            track_name: Some(filename.to_string().replace(&file_ext_with_dot, "")),
            album: None,
            artist: None,
            duration: None,
            cover_hash: None,
        });

    return Some(Track {
        title: metadata
            .track_name
            .unwrap_or(filename.to_string().replace(&file_ext_with_dot, "")),
        album: metadata.album,
        artist: metadata.artist,
        duration: metadata.duration.map(|t| t.seconds),
        path: path.clone(),
        cover_hash: metadata.cover_hash,
    });
}

struct TrackMetadata {
    track_name: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    duration: Option<Time>,
    cover_hash: Option<String>,
}

/// Get some audio metadata from the file at `path`.
fn get_audio_metadata(
    path: &PathBuf,
    file_ext: &str,
    app: &AppHandle,
) -> Result<TrackMetadata, Error> {
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
    let mut cover_hash = None;

    let mut get_tags = |metadata: &MetadataRevision| -> Result<(), Error> {
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

        for visual in metadata.visuals() {
            if let Some(usage) = visual.usage {
                match usage {
                    StandardVisualKey::FrontCover => {
                        cover_hash = save_frontcover(visual.clone(), app, &path)
                            .inspect_err(|e| {
                                warn!("Failed to save cover image for {path:?}. Error: {e}",)
                            })
                            .ok()
                    }

                    _ => (),
                }
            }
        }

        // If no front cover standard key was found and there are
        // visuals, use the first visual as the fallback cover
        if let None = cover_hash {
            if let Some(visual) = metadata.visuals().first() {
                cover_hash = save_frontcover(visual.clone(), app, &path)
                    .inspect_err(|e| warn!("Failed to save cover image for {path:?}. Error: {e}",))
                    .ok()
            }
        }

        return Ok(());
    };

    if let Some(metadata_rev) = probed.format.metadata().current() {
        get_tags(metadata_rev)?;
    } else if let Some(metadata_rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        get_tags(metadata_rev)?;
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

    return Ok(TrackMetadata {
        track_name,
        album,
        artist,
        cover_hash,
        duration,
    });
}

/// Save the image in a visual object to the app's cache after downscaling and
/// converting it. Returns the hash used as the identifier.
fn save_frontcover(visual: Visual, app: &AppHandle, path: &PathBuf) -> Result<String, Error> {
    // Hash the content of the image to make a unique filename
    let mut hasher = Sha256::new();
    hasher.update(&visual.data);
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);

    let cache_path = app.path().app_cache_dir()?;
    let extension = media_type_to_mime_type_and_ext(&visual.media_type).1;
    let cover_file = format!("{hash_str}.{extension}");

    // Make filepaths for both the cover and the thumbnail
    let relative_cover_path = format!("covers/{cover_file}");
    let absolute_cover_path = cache_path.join(relative_cover_path.clone());

    let relative_thumb_path = format!("thumbnails/{cover_file}");
    let absolute_thumb_path = cache_path.join(relative_thumb_path.clone());

    // Only do image processing if there is no cached copy already
    if !absolute_cover_path.exists() || !absolute_thumb_path.exists() {
        // Create the folders if they aren't already there
        if let Some(parent) = absolute_cover_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = absolute_thumb_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read the images by guessing their format (more reliable than using
        // the saved MIME type)
        let maybe_formatted =
            ImageReader::new(Cursor::new(visual.data.clone())).with_guessed_format();

        let formatted = match maybe_formatted {
            Ok(fmt) => fmt,
            Err(e) => {
                warn!("Could not guess format for {path:?}. Error: {e}");
                // If format guessing fails, try to use the media type tag
                let mime_type = media_type_to_mime_type_and_ext(&visual.media_type).0;
                let format = ImageFormat::from_mime_type(mime_type)
                    .expect("All supported MIME types have a format");
                let mut reader = ImageReader::new(Cursor::new(visual.data));
                reader.set_format(format);
                reader
            }
        };

        let img = formatted.decode()?;

        // Cover in downscaled to a resonable size and filtered here so that we don't
        // need to do it every time with CSS
        let mut cover = img.resize(200, 200, FilterType::CatmullRom).into_rgba8();
        cover
            .pixels_mut()
            .for_each(|p| p.apply_with_alpha(|rgb| rgb / 2, |alpha| alpha / 3));

        // Thumbnail is resized to a very small scale
        let thumbnail = img.thumbnail(64, 64);

        // Save as WebP
        cover.save_with_format(absolute_cover_path, ImageFormat::WebP)?;
        thumbnail.save_with_format(absolute_thumb_path, ImageFormat::WebP)?;
    }

    return Ok(cover_file);
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

/// Convert the media type of a visual into a pair of MIME type and correlated file extension.
fn media_type_to_mime_type_and_ext(media_type: &str) -> (String, String) {
    let media_type = media_type.to_lowercase();
    if media_type.contains("/") {
        // In theory, images that contain visuals should specify the MIME
        // type of the visual
        let ext = MIME_TYPES
            .iter()
            .find(|(mime, _ext)| *mime == media_type)
            .map(|t| t.1)
            .unwrap_or("jpg");

        return (media_type, ext.to_string());
    } else {
        // However, sometimes that's not correct and only the "media type"
        // is actually just the file extension
        let mime_type = MIME_TYPES
            .iter()
            .find(|(_mime, ext)| *ext == media_type)
            .map(|t| t.0)
            .unwrap_or("image/jpeg");

        return (mime_type.to_string(), media_type);
    }
}

const MIME_TYPES: [(&str, &str); 6] = [
    ("image/jpeg", "jpg"),
    ("image/png", "png"),
    ("image/gif", "gif"),
    ("image/bmp", "bmp"),
    ("image/webp", "webp"),
    ("image/tiff", "tiff"),
];
