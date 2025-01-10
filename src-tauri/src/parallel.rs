use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use serenity::async_trait;
use songbird::{
    events::EventData,
    tracks::{Track, TrackHandle},
    Driver, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug, Default)]
pub struct ParallelTracks {
    inner: Arc<Mutex<ParallelTracksInner>>,
}

impl ParallelTracks {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ParallelTracksInner { tracks: vec![] })),
        }
    }

    pub fn add(&self, mut track: Track, driver: &mut Driver) -> TrackHandle {
        let remote_lock = self.inner.clone();
        track.events.add_event(
            EventData::new(
                Event::Track(TrackEvent::End),
                ParallelHandler { remote_lock },
            ),
            Duration::ZERO,
        );

        let mut inner = self.inner.lock();
        let handle = driver.play(track);
        inner.tracks.push(handle.clone());

        info!("Added track in parallel.");
        return handle;
    }

    pub fn get_handle(&self, uuid: Uuid) -> Option<TrackHandle> {
        let inner = self.inner.lock();
        if let Some(track) = inner.tracks.iter().find(|t| t.uuid() == uuid) {
            return Some(track.clone());
        } else {
            return None;
        }
    }

    pub fn resume(&self, uuid: Uuid) {
        let inner = self.inner.lock();
        if let Some(track) = inner.tracks.iter().find(|t| t.uuid() == uuid) {
            drop(track.play());
        }

        info!("Resumed parallel track.")
    }

    pub fn pause(&self, uuid: Uuid) {
        let inner = self.inner.lock();
        if let Some(track) = inner.tracks.iter().find(|t| t.uuid() == uuid) {
            drop(track.pause());
        }

        info!("Paused parallel track.")
    }

    pub fn stop(&self, uuid: Uuid) {
        let inner = self.inner.lock();
        if let Some(track) = inner.tracks.iter().find(|t| t.uuid() == uuid) {
            drop(track.stop())
        }

        info!("Stopped parallel track.")
    }

    pub fn stop_all(&self) {
        let inner = self.inner.lock();
        for track in inner.tracks.iter() {
            drop(track.stop())
        }

        info!("Stopped all parallel tracks.")
    }
}

#[derive(Debug, Default)]
struct ParallelTracksInner {
    tracks: Vec<TrackHandle>,
}

struct ParallelHandler {
    remote_lock: Arc<Mutex<ParallelTracksInner>>,
}

#[async_trait]
impl EventHandler for ParallelHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();

        match ctx {
            EventContext::Track(ts) => {
                // This slice should have exactly one entry.
                // Unlike in a queue, parallel tracks can end in any order
                // so we remove the track with the correct UUID from the
                // vector.

                let ended_uuid = ts.first()?.1.uuid();
                inner.tracks.retain(|t| t.uuid() != ended_uuid);
            }
            _ => return None,
        }

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks are playing.", inner.tracks.len());

        return None;
    }
}
