use rand;
use rand::seq::IndexedRandom;
use rodio::{Decoder, OutputStream, Sink};
use std::env;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufReader, Error};
use std::path::PathBuf;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let dir_path = if args.len() == 2 { &args[1] } else { "./" };
    let dir_reader = fs::read_dir(dir_path).expect("Could not open given directory.");
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Cannot connect to audio device");
    let sink = Sink::try_new(&stream_handle).unwrap();
    let paths = dir_reader
        .collect::<Vec<Result<DirEntry, Error>>>()
        .into_iter()
        .map(|path| path.unwrap().path())
        .filter(|pb| pb.extension().is_some_and(|ext| ext == "mp3"))
        .collect::<Vec<PathBuf>>();
    if paths.is_empty() {
        panic!("Cannot run program on directory with no suitable songs.");
    }
    loop {
        let file_name = paths.choose(&mut rand::rng()).unwrap();
        let file = BufReader::new(File::open(file_name).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    }
}
