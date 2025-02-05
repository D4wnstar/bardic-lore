use parking_lot::Mutex;
use songbird::tracks::{LoopState, PlayMode};
use std::{collections::VecDeque, fs::File, io::BufReader, path::PathBuf, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::Duration;

use rodio::{Decoder, OutputStreamHandle, Sink};
use serde_json::json;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::{
    events::{QueueMethod, ADD_TRACK, TRACK_ENDED, TRACK_LOOPED, TRACK_PAUSED, TRACK_PLAYED},
    files::Track as TrackData,
};

#[derive(Debug, thiserror::Error)]
pub enum RodioError {
    #[error(transparent)]
    Stream(#[from] rodio::StreamError),
    #[error(transparent)]
    Decoder(#[from] rodio::decoder::DecoderError),
    #[error(transparent)]
    Play(#[from] rodio::PlayError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for RodioError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/* EVENT HANDLERS */
// struct RodioTrackEnd {
//     app: AppHandle,
//     uuid: String,
// }

// impl RodioTrackEnd {
//     fn new(app: AppHandle, uuid: String) -> Self {
//         Self { app, uuid }
//     }
// }

// struct RodioTrackLoop {
//     app: AppHandle,
//     uuid: String,
// }

// impl RodioTrackLoop {
//     fn new(app: AppHandle, uuid: String) -> Self {
//         Self { app, uuid }
//     }
// }

// struct RodioTrackPlay {
//     app: AppHandle,
//     uuid: String,
// }

// impl RodioTrackPlay {
//     fn new(app: AppHandle, uuid: String) -> Self {
//         Self { app, uuid }
//     }
// }

// struct RodioTrackPause {
//     app: AppHandle,
//     uuid: String,
// }

// impl RodioTrackPause {
//     fn new(app: AppHandle, uuid: String) -> Self {
//         Self { app, uuid }
//     }
// }

// impl Drop for RodioTrackEnd {
//     fn drop(&mut self) {
//         self.app
//             .emit(
//                 "track-ended",
//                 json!({
//                     "isParallel": false,
//                     "uuid": self.uuid
//                 }),
//             )
//             .unwrap();
//     }
// }

// impl Drop for RodioTrackLoop {
//     fn drop(&mut self) {
//         self.app
//             .emit(
//                 "track-looped",
//                 json!({
//                     "isParallel": false,
//                     "uuid": self.uuid
//                 }),
//             )
//             .unwrap();
//     }
// }

/// A simplified version of serenity's track object.
pub struct Track {
    /// Whether or not this sound is currently playing.
    ///
    /// Defaults to [`PlayMode::Play`].
    pub playing: PlayMode,

    /// Count of remaining loops.
    ///
    /// Defaults to play a track once (i.e., [`LoopState::Finite(0)`]).
    ///
    /// [`LoopState::Finite(0)`]: LoopState::Finite
    pub loops: LoopState,

    /// Unique identifier for this track.
    ///
    /// Defaults to a random 128-bit number.
    pub uuid: Uuid,
}

impl Track {
    /// Create a new track with a random [`Uuid`].
    #[must_use]
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();

        Self::new_with_uuid(uuid)
    }

    /// Create a new track directly with a custom [`Uuid`].
    #[must_use]
    pub fn new_with_uuid(uuid: Uuid) -> Self {
        Self {
            playing: PlayMode::default(),
            loops: LoopState::Finite(0),
            uuid,
        }
    }
}

/// A queue implementation for rodio that stores info about the tracks
/// and abstracts away the async management.
pub struct RodioQueue {
    inner: Arc<Mutex<RodioQueueCore>>,
}

impl RodioQueue {
    pub fn new(handle: &OutputStreamHandle) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RodioQueueCore::new(handle))),
        }
    }

    pub fn add(&self, path: &PathBuf) -> Result<Uuid, RodioError> {
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::new(file)?;
        let track = Track::new();
        let uuid = track.uuid.clone();

        let mut queue = self.inner.lock();
        queue.sink.append(source);
        queue.tracks.push_back(track);

        return Ok(uuid);
    }

    pub fn set_volume(&self, volume: f32) {
        let queue = self.inner.lock();
        queue.sink.set_volume(volume);
    }
}

/// The actual content of a `RodioQueue`. In the interest of similarity with the
/// Discord part of this app, this uses a simplified version of the songbird
///  `Track` struct to contain info about the tracks.
struct RodioQueueCore {
    sink: Sink,
    tracks: VecDeque<Track>,
}

impl RodioQueueCore {
    fn new(handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(handle).unwrap();
        return RodioQueueCore {
            sink,
            tracks: VecDeque::new(),
        };
    }
}

pub struct StateTracker {
    app: AppHandle,
    queue: Arc<Mutex<RodioQueueCore>>,
    current_uuid: Arc<Mutex<Option<Uuid>>>,
}

struct RodioQueueState {
    pub uuid: Option<Uuid>,
    pub playing: bool,
}

impl StateTracker {
    pub fn new(app: AppHandle, queue: &RodioQueue) -> Self {
        Self {
            app,
            queue: queue.inner.clone(),
            current_uuid: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(self) {
        tokio::spawn(async move {
            // Check for status every 100 ms
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            let mut last_state = RodioQueueState {
                uuid: None,
                playing: false,
            };

            loop {
                interval.tick().await;

                let queue = self.queue.lock();
                let curr_track = queue.tracks.front();

                let current_state = RodioQueueState {
                    uuid: curr_track.map(|t| t.uuid),
                    playing: !queue.sink.is_paused(),
                };

                // If the current UUID changed, the current track must've ended
                if current_state.uuid != last_state.uuid {
                    self.app
                        .emit(
                            TRACK_ENDED,
                            json!({
                                "isParallel": false,
                                "uuid": last_state.uuid
                            }),
                        )
                        .unwrap();
                }

                // Handle play/pause state changes
                // if current_state.0 != last_state.0 {
                //     if current_state.0 {
                //         self.app
                //             .emit(
                //                 TRACK_PAUSED,
                //                 json!({
                //                     "isParallel": false,
                //                     "uuid": current_state.1.clone()
                //                 }),
                //             )
                //             .unwrap();
                //     } else {
                //         self.app
                //             .emit(
                //                 TRACK_PLAYED,
                //                 json!({
                //                     "isParallel": false,
                //                     "uuid": current_state.1.clone()
                //                 }),
                //             )
                //             .unwrap();
                //     }
                // }

                last_state = current_state;
            }
        });
    }
}

/* TAURI COMMANDS */
#[tauri::command]
pub async fn initialize_rodio_event_handler(
    app: AppHandle,
    queue: State<'_, RodioQueue>,
) -> Result<(), RodioError> {
    // Start the state tracker for the rodio queue
    let tracker = StateTracker::new(app.clone(), queue.inner());
    tracker.start().await;

    return Ok(());
}

#[tauri::command]
pub async fn create_playlist(
    tracks: Vec<TrackData>,
    queue: State<'_, RodioQueue>,
    app: AppHandle,
) -> Result<(), RodioError> {
    // let lock = queue.lock();
    // lock.stop();

    return Ok(());
}

#[tauri::command]
pub async fn queue_track(
    track: TrackData,
    queue_method: QueueMethod,
    looping: bool,
    queue: State<'_, RodioQueue>,
    app: AppHandle,
) -> Result<(), RodioError> {
    let uuid = queue.add(&track.path)?;
    queue.set_volume(0.3);

    app.emit(
        ADD_TRACK,
        json!({
            "track": {
                "uuid": uuid,
                "title": track.title,
                "album": track.album,
                "artist": track.artist,
                "duration": track.duration,
                "path": track.path,
                "filename": track.filename,
                "coverHash": track.cover_hash,
            },
            "parallel": false,
            "queueMethod": queue_method,
            "looping": looping,
        }),
    )
    .unwrap();

    return Ok(());
}
