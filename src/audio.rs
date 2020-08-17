use lazy_static::lazy_static;

use json::{self, object};

use chrono::NaiveDateTime;

use std::sync::atomic::*;

use std::path::{Path, PathBuf};

use std::{error, fmt, fs};

use std::time::{Instant, Duration};

lazy_static! {
    static ref CURRENT_AUDIO: AtomicU64 = AtomicU64::new(0);
}

const AUDIO_UPDATE_PATH: &str = "/tmp/audio";
const AUDIO_STATUS_PATH: &str = "/tmp/audioStatus.json";
const TIME_FORMAT: &str = "%FT%T.%fZ";

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

pub struct AudioUpdate {
    volume: f64,
    paused: bool,
    does_loop: bool,
    loop_count: i64
}

fn parse_status() -> AudioResult<json::JsonValue> {
    let status_str = match fs::read_to_string(AUDIO_STATUS_PATH) {
        Ok(s) => s,
        Err(e) => Err(AudioError::new(format!("Error in reading {}. ({})", AUDIO_STATUS_PATH, e.to_string())))?
    };

    match json::parse(&status_str) {
        Ok(s) => Ok(s),
        Err(e) => Err(AudioError::new(format!("Error in parsing JSON. ({})", e.to_string())))
    }
}

fn get_status_by_id(id: u64) -> AudioResult<json::JsonValue> {
    let status = parse_status()?;

    match status["Sources"].iter().find(|&s| s["ID"] == id) {
        Some(o) => Ok(o),
        None => Err(AudioError::new(format!("No audio source found with id {}.", id)))
    }
}

fn get_status_by_name(name: &str) -> AudioResult<json::JsonValue> {
    let status = parse_status()?;

    match status["Sources"].iter().find(|&s| s["Name"] == name) {
        Some(o) => Ok(o),
        None => Err(AudioError::new(format!("No audio source found with name {}.", name)))
    }
}

impl AudioBuilder {
    pub fn new<T: AsRef<Path>>(file: T) -> Self {
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

    pub fn name<T: AsRef<str>>(self, name: T) -> Self {
        self.name = name.as_ref().to_owned();
        self
    }

    pub fn volume(self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    pub fn does_loop(self, does_loop: bool) -> Self {
        self.does_loop = does_loop;
        self
    }

    pub fn loop_count(self, loop_count: i64) -> Self {
        self.loop_count = loop_count;
        self
    }

    pub fn build(self) -> AudioResult<Audio> {
        let serialized = object! {
            Name: self.name.clone(),
            File: self.file.to_str(),
            Volume: self.volume,
            DoesLoop: self.does_loop,
            LoopCount: self.loop_count
        };

        match fs::write(AUDIO_UPDATE_PATH, serialized.dump()) {
            Ok(_) => {
                let start_time = Instant::now();
                let time_out = Duration::from_secs(2);

                while start_time.elapsed() <= time_out {
                    if let Ok(status) = get_status_by_name(&self.name) {
                        return Ok(Audio { id: status["ID"].as_u64().unwrap() });
                    }
                }

                Err(AudioError::new(format!("Timed out while waiting for {} to update.", AUDIO_UPDATE_PATH)))
            },
            Err(e) => Err(AudioError::new(format!("Error in writing to {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))
        }
    }
}

pub fn is_running() -> AudioResult<bool> {
    let status = parse_status()?;
    Ok(status["Running"].as_bool().unwrap())
}

pub fn is_disabled() -> AudioResult<bool> {
    let status = parse_status()?;
    Ok(status["Disabled"].as_bool().unwrap())
}

impl Audio {
    pub fn get_name(&self) -> AudioResult<String> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Name"].as_str().unwrap().to_owned())
    }

    pub fn get_file_type(&self) -> AudioResult<String> {
        let status = get_status_by_id(self.id)?;
        Ok(status["FileType"])
    }

    pub fn get_volume(&self) -> AudioResult<f64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Volume"])
    }
    
    pub fn get_duration(&self) -> AudioResult<u64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Duration"])
    }

    pub fn get_remaining(&self) -> AudioResult<u64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Remaining"])
    }

    pub fn is_paused(&self) -> AudioResult<bool> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Paused"])
    }

    pub fn get_loop(&self) -> AudioResult<i64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Loop"])
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_end_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status_by_id(self.id)?;

        match NaiveDateTime::parse_from_str(&status["EndTime"], TIME_FORMAT) {
            Ok(t) => Ok(t),
            Err(e) => Err(AudioError::new(format!("Error in parsing end time. ({})", e.to_string())))
        }
    }

    pub fn get_start_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status_by_id(self.id)?;

        match NaiveDateTime::parse_from_str(&status["StartTime"], TIME_FORMAT) {
            Ok(t) => Ok(t),
            Err(e) => Err(AudioError::new(format!("Error in parsing start time. ({})", e.to_string())))
        }
    }

    pub fn update(&mut self, update: AudioUpdate) -> AudioResult<()> {
        let serialized = object! {
            ID: self.id,
            Volume: update.volume,
            Paused: update.paused,
            DoesLoop: update.does_loop,
            LoopCount: update.loop_count
        };

        match fs::write(AUDIO_UPDATE_PATH, serialized.dump()) {
            Ok(_) => Ok(()),
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

