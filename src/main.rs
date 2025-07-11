use axum::{
    extract::{Query, State},
    routing::put,
    Router,
};
use rand;
use rand::seq::IndexedRandom;
use rodio::{Decoder, OutputStream, Sink};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufReader, Error};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env};
use tokio;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();
    let dir_path = if args.len() == 2 { &args[1] } else { "./" };
    let dir_reader = fs::read_dir(dir_path).expect("Could not open given directory.");
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Cannot connect to audio device");
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    let paths = dir_reader
        .collect::<Vec<Result<DirEntry, Error>>>()
        .into_iter()
        .map(|path| path.unwrap().path())
        .filter(|pb| pb.extension().is_some_and(|ext| ext == "mp3"))
        .collect::<Vec<PathBuf>>();
    if paths.is_empty() {
        panic!("Cannot run program on directory with no suitable songs.");
    }
    let server_sink = sink.clone();
    tokio::spawn(async move {
        process(server_sink).await;
    });
    loop {
        let file_name = paths.choose(&mut rand::rng()).unwrap();
        let file = BufReader::new(File::open(file_name).unwrap());
        let source = Decoder::new(file).unwrap();
        {
            let sink = sink.lock().unwrap();
            sink.append(source);
        }
        while !sink.lock().unwrap().empty() {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

struct AppState {
    sink: Arc<Mutex<Sink>>,
}

async fn process(sink: Arc<Mutex<Sink>>) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    let shared_state = Arc::new(AppState { sink });
    let app = Router::new()
        .route("/volume", put(change_volume))
        .route("/pause", put(pause))
        .route("/resume", put(resume))
        .with_state(shared_state);
    axum::serve(listener, app).await.unwrap();
}

async fn change_volume(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> &'static str {
    if let Some(new_volume_str) = params.get("v") {
        if let Ok(new_volume) = new_volume_str.parse::<u32>() {
            if new_volume <= 100 {
                let scaled_new_volume = new_volume as f32 / 100.0f32;
                let sink = state.sink.lock().unwrap();
                sink.set_volume(scaled_new_volume);
                "Volume changed successfully"
            } else {
                "Volume must be between 0 and 100"
            }
        } else {
            "Volume must be an integer between 1 and 100"
        }
    } else {
        "You must specify the volume"
    }
}

async fn pause(State(state): State<Arc<AppState>>) -> &'static str {
    let sink = state.sink.lock().unwrap();
    sink.pause();
    "Paused successfully"
}

async fn resume(State(state): State<Arc<AppState>>) -> &'static str {
    let sink = state.sink.lock().unwrap();
    sink.play();
    "Resumed successfully"
}
