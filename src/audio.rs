//! Everything you need to play audio in repl.it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::{error, fmt, fs};
use std::io::Write;
use std::time::{Instant, Duration};

use lazy_static::lazy_static;
use json::{self, object};
use chrono::NaiveDateTime;

lazy_static! {
    static ref CURRENT_AUDIO: AtomicU64 = AtomicU64::new(0);
}

const AUDIO_UPDATE_PATH: &str = "/tmp/audio";
const AUDIO_STATUS_PATH: &str = "/tmp/audioStatus.json";
const TIME_FORMAT: &str = "%FT%T.%fZ"; // yyyy-mm-ddThh-mm-ss.sssssssssZ

/// Used to play an audio file or tone and create an `Audio` instance.
pub struct AudioBuilder {
    name: Option<String>,
    audio_type: AudioType,
    volume: f64,
    does_loop: bool,
    loop_count: i64
}

/// A struct providing access to some currently playing audio instance.
pub struct Audio {
    id: u64,
    audio_type: AudioType
}

/// A struct with fields for updating a currently playing audio instance.
pub struct AudioUpdate {
    pub volume: f64,
    pub paused: bool,
    pub does_loop: bool,
    /// Use a negative loop count to indicate an infinite loop.
    pub loop_count: i64
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
    let mut status = parse_status()?;

    match status["Sources"].members_mut().find(|s| s["ID"] == id) {
        Some(o) => Ok(o.take()),
        None => Err(AudioError::new(format!("No audio source found with id {}.", id)))
    }
}

fn get_status_by_name(name: &str) -> AudioResult<json::JsonValue> {
    let mut status = parse_status()?;

    match status["Sources"].members_mut().find(|s| s["Name"] == name) {
        Some(o) => Ok(o.take()),
        None => Err(AudioError::new(format!("No audio source found with name {}.", name)))
    }
}

impl AudioBuilder {
    /// Create a new `AudioBuilder` with a certain `AudioType`, which describes either an
    /// audio file or a tone.
    pub fn new(audio_type: &AudioType) -> Self {
        AudioBuilder {
            name: None,
            audio_type: audio_type.clone(),
            volume: 1.0,
            does_loop: false,
            loop_count: -1
        }
    }

    /// Set the name of the audio instance. Each audio instance should have a unique name.
    ///
    /// By default, a unique name is created for each audio instance.
    /// Setting a custom name is *not* recommended.
    pub fn name<T: AsRef<str>>(mut self, name: T) -> Self {
        self.name = Some(name.as_ref().to_owned());
        self
    }

    /// Set the volume of the audio instance.
    ///
    /// By default, the volume is `1.0`.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set whether the audio instance loops.
    ///
    /// By default, the audio instance does not loop.
    pub fn does_loop(mut self, does_loop: bool) -> Self {
        self.does_loop = does_loop;
        self
    }

    /// Set the number of times to loop.
    ///
    /// `does_loop` should be used to allow looping of the audio instance.
    /// By default, the audio instance loops forever.
    pub fn loop_count(mut self, loop_count: i64) -> Self {
        self.loop_count = loop_count;
        self
    }

    /// Play the audio instance and return an `Audio` struct, which can be used to get or
    /// update properties of the audio instance.
    ///
    /// This can be called multiple times to play a certain audio multiple times.
    /// This will block until the audio instance begins playing.
    pub fn build(&self) -> AudioResult<Audio> {
        let name = match &self.name {
            Some(n) => n.to_owned(),
            // generate unique name
            None => format!("rust_audio_{}", CURRENT_AUDIO.fetch_add(1, Ordering::SeqCst))
        };

        let serialized_args = match self.audio_type {
            AudioType::File { ref path, .. } => object! {
                Path: path.as_str()
            },
            AudioType::Tone { tone, pitch, duration } => object! {
                WaveType: tone as u8,
                Pitch: pitch,
                Seconds: duration
            }
        };

        let serialized = object! {
            Name: name.as_str(),
            Type: self.audio_type.as_str(),
            Volume: self.volume,
            DoesLoop: self.does_loop,
            LoopCount: self.loop_count,
            Args: serialized_args
        };

        let mut file = match fs::OpenOptions::new().append(true).open(AUDIO_UPDATE_PATH) {
            Ok(f) => f,
            Err(e) => Err(AudioError::new(format!("Error in opening {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))?
        };

        match write!(&mut file, "{}", serialized.dump()) {
            Ok(_) => {
                let start_time = Instant::now();
                let time_out = Duration::from_secs(2);

                while start_time.elapsed() <= time_out {
                    if let Ok(status) = get_status_by_name(&name) {
                        return Ok(Audio { id: status["ID"].as_u64().unwrap(), audio_type: self.audio_type.clone() });
                    }
                }

                Err(AudioError::new(format!("Timed out while waiting for {} to update.", AUDIO_STATUS_PATH)))
            },
            Err(e) => Err(AudioError::new(format!("Error in writing to {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))
        }
    }
}

/// Get whether there are any audio instances playing.
pub fn is_running() -> AudioResult<bool> {
    let status = parse_status()?;
    Ok(status["Running"].as_bool().unwrap())
}

/// Get whether the repl.it audio playing program is running.
pub fn is_disabled() -> AudioResult<bool> {
    let status = parse_status()?;
    Ok(status["Disabled"].as_bool().unwrap())
}

impl Audio {
    /// Get the name of the audio instance.
    pub fn get_name(&self) -> AudioResult<String> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Name"].as_str().unwrap().to_owned())
    }

    /// Get the `AudioType` of the audio instance.
    pub fn get_type(&self) -> AudioType {
        self.audio_type.clone()
    }

    /// Get the volume level of the audio instance.
    pub fn get_volume(&self) -> AudioResult<f64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Volume"].as_f64().unwrap())
    }

    /// Get the duration (in milliseconds) of the audio instance.
    pub fn get_duration(&self) -> AudioResult<u64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Duration"].as_u64().unwrap())
    }

    /// Get the remaining time (in milliseconds) of the audio instance.
    pub fn get_remaining(&self) -> AudioResult<u64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Remaining"].as_u64().unwrap())
    }

    /// Get whether the audio instance is paused.
    pub fn is_paused(&self) -> AudioResult<bool> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Paused"].as_bool().unwrap())
    }

    /// Get the number of times the audio instance will loop.
    pub fn get_loop(&self) -> AudioResult<i64> {
        let status = get_status_by_id(self.id)?;
        Ok(status["Loop"].as_i64().unwrap())
    }

    /// Get the ID of the audio instance.
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Get the end time of the audio instance.
    pub fn get_end_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status_by_id(self.id)?;

        match NaiveDateTime::parse_from_str(status["EndTime"].as_str().unwrap(), TIME_FORMAT) {
            Ok(t) => Ok(t),
            Err(e) => Err(AudioError::new(format!("Error in parsing end time. ({})", e.to_string())))
        }
    }

    /// Get the start time of the audio instance.
    pub fn get_start_time(&self) -> AudioResult<NaiveDateTime> {
        let status = get_status_by_id(self.id)?;

        match NaiveDateTime::parse_from_str(status["StartTime"].as_str().unwrap(), TIME_FORMAT) {
            Ok(t) => Ok(t),
            Err(e) => Err(AudioError::new(format!("Error in parsing start time. ({})", e.to_string())))
        }
    }

    /// Update the audio instance by using the `AudioUpdate` struct.
    pub fn update(&mut self, update: &AudioUpdate) -> AudioResult<()> {
        let serialized = object! {
            ID: self.id,
            Volume: update.volume,
            Paused: update.paused,
            DoesLoop: update.does_loop,
            LoopCount: update.loop_count
        };

        let mut file = match fs::OpenOptions::new().append(true).open(AUDIO_UPDATE_PATH) {
            Ok(f) => f,
            Err(e) => Err(AudioError::new(format!("Error in opening {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))?
        };

        match write!(&mut file, "{}", serialized.dump()) {
            Ok(_) => Ok(()),
            Err(e) => Err(AudioError::new(format!("Error in writing to {}. ({})", AUDIO_UPDATE_PATH, e.to_string())))
        }
    }
}

/// Custom result type for playing audio.
pub type AudioResult<T> = Result<T, AudioError>;

/// Custom error type for audio errors.
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

/// Type of audio to play: audio file or tone.
#[derive(Debug, PartialEq, Clone)]
pub enum AudioType {
    File { file: FileType, path: String },
    Tone { tone: ToneType, pitch: f64, duration: f64 }
}

impl AudioType {
    fn as_str(&self) -> &'static str {
        match self {
            AudioType::File { file, .. } => file.as_str(),
            AudioType::Tone { .. } => "tone"
        }
    }
}

/// Supported audio file formats.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FileType {
    Wav,
    Aiff,
    Mp3
}

impl FileType {
    fn as_str(&self) -> &'static str {
        match self {
            FileType::Wav => "wav",
            FileType::Aiff => "aiff",
            FileType::Mp3 => "mp3"
        }
    }
}

/// Supported tone types.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ToneType {
    Sine = 0,
    Triangle = 1,
    Saw = 2,
    Square = 3
}

