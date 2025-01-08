use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use serenity::async_trait;
use songbird::{
    events::EventData,
    tracks::{Track, TrackHandle},
    Driver, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::info;

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
        info!("Added track in parallel.");

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

        return handle;
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

/// Extension trait to implement the parallel store functions into the songbird Driver.
pub trait Parallel {
    fn add_parallel_track(&mut self, track: Track, parallel_tracks: &ParallelTracks)
        -> TrackHandle;
}

impl Parallel for Driver {
    fn add_parallel_track(
        &mut self,
        track: Track,
        parallel_tracks: &ParallelTracks,
    ) -> TrackHandle {
        parallel_tracks.add(track, self)
    }
}
