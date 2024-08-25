use std::{fs::File, io::BufReader, sync::Mutex};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tauri::State;

/// Wrapper around `rodio::OutputStream` for thread-safety. Necessary to
/// add it to the global Tauri State manager (`cpal::Stream` isn't `Send`).
struct MutexOutputStream {
    _inner: Mutex<OutputStream>,
}

unsafe impl Send for MutexOutputStream {}
unsafe impl Sync for MutexOutputStream {}

impl MutexOutputStream {
    fn new(stream: OutputStream) -> Self {
        return MutexOutputStream {
            _inner: Mutex::new(stream),
        };
    }
}

/// Global reference to a `MutexOutputStream` and its `OutputStreamHandle`. Keeps
/// the OutputStream valid for the entire lifetime of the app.
pub struct GlobalStream {
    _stream: MutexOutputStream,
    pub stream_handle: OutputStreamHandle,
}

impl Default for GlobalStream {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        return GlobalStream {
            _stream: MutexOutputStream::new(stream),
            stream_handle,
        };
    }
}

/// A thread-safe, mutable list of all available Sinks.
pub struct SinkList {
    pub sinks: Mutex<Vec<Sink>>,
}

impl Default for SinkList {
    fn default() -> Self {
        return SinkList {
            sinks: Mutex::new(vec![]),
        };
    }
}

#[tauri::command]
pub fn play(stream: State<GlobalStream>, sinklist_state: State<SinkList>) {
    println!("Starting playback");
    let file = BufReader::new(
        File::open("../test_audio/Soundboards/Dungeon SoundPad/Door Close.ogg").unwrap(),
    );
    let source = Decoder::new(file).unwrap();

    let mut sinklist = sinklist_state.sinks.lock().unwrap();

    if sinklist.len() == 0 {
        sinklist.push(Sink::try_new(&stream.stream_handle).unwrap());
    }

    let sink = &sinklist[0];
    if sink.len() == 0 {
        sink.append(source);
    }
    sink.play()
}
