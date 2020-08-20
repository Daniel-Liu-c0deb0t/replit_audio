//! Rust library for playing audio in repl.it.
//!
//! Provides an `AudioBuilder` struct that allows the user to build and
//! play an audio file or tone. Each audio instance can be manipulated
//! with the `Audio` struct.

pub mod audio;

// re-export the functions and structs in the audio file
pub use audio::*;
