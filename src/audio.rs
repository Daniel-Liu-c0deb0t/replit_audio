use lazy_static::lazy_static;

use json::{self, object};

use chrono::NaiveDateTime;

use std::sync::atomic::*;

use std::path::Path;

use std::{error, fmt, fs};

lazy_static! {
    static ref CURRENT_AUDIO: AtomicU64 = AtomicU64::new(0);
}

// TODO: wrap audio stuff in mutex
// Cache ID for speed
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
    name: String
}

pub struct AudioUpdate {
    volume: f64,
    paused: bool,
    does_loop: bool,
    loop_count: i64
}

pub impl AudioBuilder {
    fn new<T: AsRef<Path>>(file: T) -> Self {
        // generate a unique name
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

    fn build(self) -> AudioResult<Audio> {
        let serialized = object! {
            Name: self.name.clone(),
            File: self.file,
            Volume: self.volume,
            DoesLoop: self.does_loop,
            LoopCount: self.loop_count
        };

        match fs::write(AUDIO_UPDATE_PATH, serialized.dump()) {
            Ok => Ok(Audio { self.name }),
            Err(e) => Err(AudioError::new(format!("Error in writing to {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))
        }
    }
}

fn get_status(name: &str) -> AudioResult<json::JsonObject> {
    let status_str = match fs::read_to_string(AUDIO_STATUS_PATH) {
        Ok(s) => s,
        Err(e) => Err(AudioError::new(format!("Error in reading {}. ({})", AUDIO_STATUS_PATH, e.to_string())))
    }

    let status = json::parse(status_str).expect("Error in JSON parsing.");

    match status["Sources"].iter().find(|&s| s["Name"] == name) {
        Some(o) => Ok(o),
        None => Err(AudioError::new(format!("No audio source found with name {}.", name)))
    }
}

pub impl Audio {
    fn get_name(&self) -> AudioResult<String> {
        let status = get_status(&self.name)?;
        Ok(status["Name"])
    }

    fn get_file_type(&self) -> AudioResult<String> {
        let status = get_status(&self.name)?;
        Ok(status["FileType"])
    }

    fn get_volume(&self) -> AudioResult<f64> {
        let status = get_status(&self.name)?;
        Ok(status["Volume"])
    }
    
    fn get_duration(&self) -> AudioResult<u64> {
        let status = get_status(&self.name)?;
        Ok(status["Duration"])
    }

    fn get_remaining(&self) -> AudioResult<u64> {
        let status = get_status(&self.name)?;
        Ok(status["Remaining"])
    }

    fn is_paused(&self) -> AudioResult<bool> {
        let status = get_status(&self.name)?;
        Ok(status["Paused"])
    }

    fn get_loop(&self) -> AudioResult<i64> {
        let status = get_status(&self.name)?;
        Ok(status["Loop"])
    }

    fn get_id(&self) -> AudioResult<String> {
        let status = get_status(&self.name)?;
        Ok(status["ID"])
    }

    fn get_end_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status(&self.name)?;
        Ok(status["EndTime"])
    }

    fn get_start_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status(&self.name)?;
        Ok(status["StartTime"])
    }

    fn update(&mut self, update: AudioUpdate) -> AudioResult<()> {
        let serialized = object! {
            ID: self.get_id(),
            Volume: update.volume,
            Paused: update.paused,
            DoesLoop: update.does_loop,
            LoopCount: update.loop_count
        };

        match fs::write(AUDIO_UPDATE_PATH, serialized.dump()) {
            Ok => Ok(()),
            Err(e) => Err(AudioError::new(format!("Error in writing to {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))
        }
    }
}

type AudioResult<T> = Result<T, AudioError>;

#[derive(Debug)]
pub struct AudioError {
    msg: String
}

impl AudioError {
    fn new(msg: String) -> AudioError {
        AudioError { msg: msg }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for AudioError {
    fn description(&self) -> &str {
        &self.msg
    }
}

