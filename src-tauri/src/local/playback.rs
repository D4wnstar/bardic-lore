use tauri::{AppHandle, State};

use crate::{
    events::QueueMethod, files::Track as TrackData, local::sink::RodioError,
    playback::discord::TrackAction,
};

use super::sink::RodioQueue;

/* TAURI COMMANDS */
#[tauri::command]
pub async fn create_playlist(
    tracks_data: Vec<TrackData>,
    volume: f32,
    loop_first: bool,
    shuffle: bool,
    queue: State<'_, RodioQueue>,
) -> Result<(), RodioError> {
    if tracks_data.is_empty() {
        return Ok(());
    }

    queue.stop();
    queue.set_volume(volume);
    queue.add_many(&tracks_data, loop_first)?;

    return Ok(());
}

#[tauri::command]
pub async fn queue_track(
    track_data: TrackData,
    queue_method: QueueMethod,
    looping: bool,
    volume: f32,
    queue: State<'_, RodioQueue>,
) -> Result<(), RodioError> {
    queue.set_volume(volume);
    queue.add(&track_data, looping)?;

    return Ok(());
}

/// Mandatory if `parallel` is true
/// Mandatory for a seek action
/// Mandatory for a change volume action
/// Mandatory for a sort action
#[tauri::command]
pub async fn queue_action(
    action: TrackAction,
    parallel: bool,
    uuid: Option<String>,
    position: Option<u64>,
    volume: Option<f32>,
    sort_uuids: Option<Vec<String>>,
    queue: State<'_, RodioQueue>,
    app: AppHandle,
) -> Result<(), RodioError> {
    match action {
        TrackAction::Resume => queue.resume(),
        TrackAction::Pause => queue.pause(),
        TrackAction::Skip => queue.skip(),
        TrackAction::Seek => {
            queue.try_seek(position.expect("There should a position to seek to"))?
        }
        _ => println!("Unimplemented"),
    };

    return Ok(());
}
