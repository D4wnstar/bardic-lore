use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use parking_lot::Mutex;
use rodio::{
    source::{EmptyCallback, SeekError},
    Decoder, OutputStreamHandle, Sample, Source,
};
use serde_json::json;
use songbird::tracks::LoopState;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{
    events::{
        QueueMethod, ADD_TRACK, PLAYLIST_CREATED, TRACK_ENDED, TRACK_PAUSED, TRACK_PLAYABLE,
        TRACK_PLAYED, UPDATE_PLAYER,
    },
    files::Track as TrackData,
};

use super::queue::{SourcesQueueInput, SourcesQueueOutput};

#[derive(Debug, thiserror::Error)]
pub enum RodioError {
    #[error(transparent)]
    Stream(#[from] rodio::StreamError),
    #[error(transparent)]
    Decoder(#[from] rodio::decoder::DecoderError),
    #[error(transparent)]
    Play(#[from] rodio::PlayError),
    #[error(transparent)]
    Seek(#[from] rodio::source::SeekError),
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

/// A simplified version of serenity's Track object.
#[derive(Debug, Clone)]
pub struct Track {
    /// Unique identifier for this track.
    ///
    /// Defaults to a random 128-bit number.
    pub uuid: Uuid,

    pub path: PathBuf,
}

impl Track {
    /// Create a new track with a random [`Uuid`].
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        let uuid = Uuid::new_v4();

        Self::new_with_uuid(uuid, path)
    }

    /// Create a new track directly with a custom [`Uuid`].
    #[must_use]
    pub fn new_with_uuid(uuid: Uuid, path: PathBuf) -> Self {
        Self { uuid, path }
    }
}

/// A custom implementation of a [`rodio::Sink`]. In the interest of similarity with the
/// Discord part of this app, each track is given a UUID to be identified.
pub struct RodioQueue {
    sources: Arc<SourcesQueueInput<f32>>,
    state: Arc<QueueState>,
    tracks: Arc<Mutex<VecDeque<Track>>>,
    sleep_until_end: Mutex<Option<Receiver<()>>>,
    app: AppHandle,
}

/// The state of the [`RodioQueue`].
struct QueueState {
    paused: AtomicBool,
    volume: Mutex<f32>,
    stopped: AtomicBool,
    to_clear: Mutex<u64>,
    position: Mutex<u64>,
    seek: Mutex<Option<SeekOrder>>,
}

struct SeekOrder {
    pos: Duration,
    feedback: Sender<Result<(), SeekError>>,
}

impl SeekOrder {
    fn new(pos: Duration) -> (Self, Receiver<Result<(), SeekError>>) {
        let (tx, rx) = std::sync::mpsc::channel();

        return (Self { pos, feedback: tx }, rx);
    }

    fn attempt<S>(self, maybe_seekable: &mut S)
    where
        S: Source,
        S::Item: Sample + Send,
    {
        let res = maybe_seekable.try_seek(self.pos);
        let _ignore_receiver_dropped = self.feedback.send(res);
    }
}

// Inside the source code of rodio, there is an internal signal passing system
// to notify the queue (i.e. the sink) when a source ends. Unfortunately, there is
// no way to hook into it directly. This is a known annoyance and there is a
// tracking issue for it here: https://github.com/RustAudio/rodio/issues/619.
// What is available is the EmptyCallback source, which is essentially a "fake"
// sound that runs a function when it is played. We can use these to manually
// emit a TRACK_ENDED signal when a track ends by adding them after every track.
// This means that the length of the queue is actually twice the number of tracks,
// since each track needs to be paired with an EmptyCallback.
impl RodioQueue {
    pub fn new(app: AppHandle) -> (Self, SourcesQueueOutput<f32>) {
        let (queue_input, queue_output) = super::queue::queue(true);
        let queue = RodioQueue {
            sources: queue_input,
            state: Arc::new(QueueState {
                paused: AtomicBool::new(false),
                volume: Mutex::new(1.0),
                stopped: AtomicBool::new(false),
                to_clear: Mutex::new(0),
                position: Mutex::new(0),
                seek: Mutex::new(None),
            }),
            tracks: Arc::new(Mutex::new(VecDeque::new())),
            sleep_until_end: Mutex::new(None),
            app,
        };

        return (queue, queue_output);
    }

    pub fn new_and_play(app: AppHandle, handle: &OutputStreamHandle) -> Self {
        let (queue, queue_output) = RodioQueue::new(app.clone());
        handle.play_raw(queue_output).unwrap();
        return queue;
    }

    fn setup_new_track(
        &self,
        path: &PathBuf,
    ) -> Result<(impl Source<Item = f32>, Track, EmptyCallback<f32>), RodioError> {
        let state = self.state.clone();
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::new(file)?
            // must be placed before pausable but after speed & delay
            .track_position()
            .amplify(1.0)
            .pausable(false)
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_millis(5), move |stoppable| {
                // Every 5 ms, the source...
                // ...checks if it needs to stop
                if state.stopped.load(Ordering::SeqCst) {
                    stoppable.stop();
                    *state.position.lock() = 0;
                }

                // ...checks if it needs to be skipped
                {
                    let skippable = stoppable.inner_mut();
                    let mut to_clear = state.to_clear.lock();
                    if *to_clear > 0 {
                        skippable.skip();
                        *to_clear -= 1;
                        *state.position.lock() = 0;
                    } else {
                        let track_pos = skippable.inner().inner().inner();
                        *state.position.lock() = track_pos.get_pos().as_secs();
                    }
                }

                // ...checks if it needs to be paused
                let pausable = stoppable.inner_mut().inner_mut();
                pausable.set_paused(state.paused.load(Ordering::SeqCst));

                // ...update the volume
                let amp = pausable.inner_mut();
                amp.set_factor(*state.volume.lock());

                // ...if a seek attempt needs to be made
                if let Some(seek) = state.seek.lock().take() {
                    seek.attempt(stoppable);
                }
            })
            .convert_samples::<f32>();

        let track = Track::new(path.clone());
        let uuid = track.uuid.clone();

        // An ArcMutex of the queue is passed to each callback to modify it on track end
        let app_handle = self.app.clone();
        let tracks = self.tracks.clone();
        let callback = EmptyCallback::<f32>::new(Box::new(move || {
            tracks.lock().pop_front();
            app_handle
                .emit(TRACK_ENDED, json!({ "isParallel": false, "uuid": uuid }))
                .unwrap();
        }));

        return Ok((source, track, callback));
    }

    fn make_track_json(&self, uuid: Uuid, track_data: &TrackData) -> serde_json::Value {
        json!({
            "uuid": uuid,
            "title": track_data.title,
            "album": track_data.album,
            "artist": track_data.artist,
            "duration": track_data.duration,
            "path": track_data.path,
            "filename": track_data.filename,
            "coverHash": track_data.cover_hash,
        })
    }

    pub fn add(&self, track_data: &TrackData, looping: bool) -> Result<Uuid, RodioError> {
        let (source, track, callback) = self.setup_new_track(&track_data.path)?;
        let uuid = track.uuid.clone();

        // Wait for queue to flush then resume stopped playback
        if self.state.stopped.load(Ordering::SeqCst) {
            if self.tracks.lock().len() > 0 {
                self.sleep_until_end();
            }
            self.state.stopped.store(false, Ordering::SeqCst);
        }

        let recv = self.sources.append_with_signal(source, false);
        self.sources.append(callback, true);
        self.tracks.lock().push_back(track);
        *self.sleep_until_end.lock() = Some(recv);

        self.resume();
        let track_json = self.make_track_json(uuid, &track_data);
        self.app
            .emit(
                ADD_TRACK,
                json!({
                    "track": track_json,
                    "parallel": false,
                    "queueMethod": QueueMethod::Normal,
                    "looping": looping,
                }),
            )
            .unwrap();

        return Ok(uuid);
    }

    pub fn add_many(
        &self,
        tracks_data: &Vec<TrackData>,
        loop_first: bool,
    ) -> Result<Vec<Uuid>, RodioError> {
        // Wait for queue to flush then resume stopped playback
        if self.state.stopped.load(Ordering::SeqCst) {
            if self.tracks.lock().len() > 0 {
                self.sleep_until_end();
                self.tracks.lock().drain(..);
            }
            self.state.stopped.store(false, Ordering::SeqCst);
        }

        let mut uuids = vec![];
        let mut response_tracks = vec![];
        let mut recv = None;
        for (idx, track_data) in tracks_data.iter().enumerate() {
            let (source, track, callback) = self.setup_new_track(&track_data.path)?;
            let uuid = track.uuid.clone();
            let looping = if idx == 0 { loop_first } else { false };

            if idx < tracks_data.len() - 1 {
                self.sources.append(source.convert_samples(), false);
            } else {
                recv = Some(
                    self.sources
                        .append_with_signal(source.convert_samples(), false),
                );
            }
            self.sources.append(callback, true);
            self.tracks.lock().push_back(track);

            uuids.push(uuid);
            response_tracks.push(self.make_track_json(uuid, &track_data));
        }

        *self.sleep_until_end.lock() = recv;
        self.resume();

        let out = serde_json::to_value(&response_tracks).unwrap();
        let res = json!({ "tracks": out });
        self.app.emit(PLAYLIST_CREATED, res).unwrap();
        // Unlike on Discord, where tracks need to be streamed, local tracks are playable immediately
        self.app
            .emit(
                TRACK_PLAYABLE,
                json!({ "isParallel": false, "uuid": response_tracks[0]}),
            )
            .unwrap();

        return Ok(uuids);
    }

    // TODO: Figure out if there's a way to prepend a track in a rodio Sink
    // pub fn prepend(&self, track_data: &TrackData, looping: bool) -> Result<Uuid, RodioError> {
    //     let (source, track, callback) = self.setup_new_track(&track_data.path)?;
    //     let uuid = track.uuid.clone();

    //     let mut queue = self.inner.lock();

    //     return Ok(uuid);
    // }

    pub fn set_volume(&self, volume: f32) {
        *self.state.volume.lock() = volume;
    }

    pub fn stop(&self) {
        self.state.stopped.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        // In serenity, play/pause state is per track, so we can't play an empty queue
        // To have that same behavior, only play if the queue is not empty
        if let Some(current) = self.tracks.lock().front() {
            self.state.paused.store(false, Ordering::SeqCst);

            // Since events are not fired automatically, manually fire a TRACK_PLAYED
            self.app
                .emit(
                    TRACK_PLAYED,
                    json!({ "isParallel": false, "uuid": current.uuid }),
                )
                .unwrap();
        };
    }

    pub fn pause(&self) {
        // See resume comment
        if let Some(current) = self.tracks.lock().front() {
            self.state.paused.store(true, Ordering::SeqCst);

            // Since events are not fired automatically, manually fire a TRACK_PAUSED
            self.app
                .emit(
                    TRACK_PAUSED,
                    json!({ "isParallel": false, "uuid": current.uuid }),
                )
                .unwrap();
        };
    }

    pub fn skip(&self) {
        let mut to_clear = self.state.to_clear.lock();
        let number_of_tracks = self.tracks.lock().len();
        if number_of_tracks as u64 > *to_clear {
            *to_clear += 1;
        };
        // The track's callback will run immediately after it is skipped
    }

    pub fn try_seek(&self, position: u64) -> Result<(), rodio::source::SeekError> {
        let (order, feedback) = SeekOrder::new(Duration::from_secs(position));
        *self.state.seek.lock() = Some(order);

        if self.tracks.lock().len() == 0 {
            // No sound is playing, seek will not be performed
            return Ok(());
        }

        return match feedback.recv() {
            Ok(Ok(())) => {
                *self.state.position.lock() = position;
                self.app
                    .emit(UPDATE_PLAYER, json!({ "position": position }))
                    .unwrap();
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::warn!("{e}");
                // If seeking returns an error, we delete the source and rebuild it,
                // then take_duration until the correct position
                // We are currently playing Track A and the
                // the current queue looks something like
                // Callback A - Track B - Callback B - ...
                // We want to prepend A
                // Track A* - Callback A - Track B - Callback B - ...
                // Then skip the current track to end it
                // (inserting modifies the queue input, skip modifies the queue output)
                // This guarantees that the callback isn't accidentally triggered

                let (source, _, _) = {
                    let current_track = &self.tracks.lock()[0];
                    self.setup_new_track(&current_track.path).unwrap()
                };

                println!("Before skipping duration");
                let source = source.skip_duration(Duration::from_secs(position));
                println!("After skipping duration");
                // println!("Before: {:?}", self.sources);
                self.sources.insert(0, source, false);
                // println!("After: {:?}", self.sources);
                self.skip();
                self.app
                    .emit(UPDATE_PLAYER, json!({ "position": position }))
                    .unwrap();

                Ok(())
            }
            // The feedback channel closed. Probably another SeekOrder was set
            // invalidating this one and closing the feedback channel
            // ... or the audio thread panicked.
            Err(_) => Ok(()),
        };
    }

    pub fn sleep_until_end(&self) {
        if let Some(sleep_until_end) = self.sleep_until_end.lock().take() {
            let _ = sleep_until_end.recv();
        };
    }
}
