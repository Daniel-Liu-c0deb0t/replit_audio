use lazy_static::lazy_static;

use json::{self, object};

use std::sync::atomic::*;

use std::path::Path;

use std::fs;

lazy_static! {
    static ref CURRENT_AUDIO: AtomicU64 = AtomicU64::new(0);
}

const AUDIO_UPDATE_PATH: &str = "/tmp/audio";
const AUDIO_STATUS_PATH: &str = "/tmp/audioStatus.json";

pub struct AudioBuilder {
    name: String,
    file: PathBuf,
    volume: f64,
    does_loop: bool,
    loop_count: i64
}

pub struct Audio {
    id: u64
}

pub impl AudioBuilder {
    fn new<T: AsRef<Path>>(file: T) -> Self {
        // temporary, unique name
        let name = format!("rust_audio_{}", CURRENT_AUDIO.load(Ordering::SeqCst));
        CURRENT_AUDIO.fetch_add(1, Ordering::SeqCst);

        AudioBuilder {
            name: name,
            file: file.as_ref().to_owned(),
            volume: 1.0,
            does_loop: false,
            loop_count: -1
        }
    }

    fn name<T: AsRef<str>>(self, name: T) -> Self {
        self.name = name.as_ref().to_owned();
        self
    }

    fn volume(self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    fn does_loop(self, does_loop: bool) -> Self {
        self.does_loop = does_loop;
        self
    }

    fn loop_count(self, loop_count: i64) -> Self {
        self.loop_count = loop_count;
        self
    }

    fn build(self) -> Audio {
        let serialized = object! {
            Name: self.name.clone(),
            File: self.file,
            Volume: self.volume,
            DoesLoop: self.does_loop,
            LoopCount: self.loop_count
        };

        fs::write(AUDIO_UPDATE_PATH, serialized.stringify())
            .expect(format!("Error in writing to {}.", AUDIO_UPDATE_PATH).as_str());

        let status_str = fs::read_to_string(AUDIO_STATUS_PATH)
            .expect(format!("Error in reading {}.", AUDIO_STATUS_PATH).as_str());

        let status = json::parse(status_str).expect("Error in JSON parsing.");

        status["Sources"].iter().find(|&s| s["Name"] == self.name);
    }
}

pub impl Audio {

}
