use replit_audio::*;

use std::thread;
use std::time::Duration;

#[test]
fn test() {
    // this test should be conducted on repl.it
    // first, play a tone, then turn down the volume halfway through
    test_play_tone();
    thread::sleep(Duration::from_secs(2));
    // finally, play 30 seconds of the mysterious audio file
    test_play_audio_file();
    thread::sleep(Duration::from_secs(30));
}

fn test_play_audio_file() {
    let audio = AudioBuilder::new(&AudioType::File { file: FileType::Wav, path: "audio.wav".to_string() })
        .volume(1.0)
        .does_loop(true)
        .loop_count(-1)
        .build()
        .unwrap();

    assert_eq!(audio.get_volume().unwrap(), 1.0);
    assert_eq!(audio.get_loop().unwrap(), -1);
    audio.get_duration().unwrap();
    audio.get_remaining().unwrap();
    audio.get_start_time().unwrap();
    audio.get_end_time().unwrap();
    audio.is_paused().unwrap();

    assert_eq!(replit_audio::is_disabled().unwrap(), false);
    assert_eq!(replit_audio::is_running().unwrap(), true);
}

fn test_play_tone() {
    let mut audio = AudioBuilder::new(&AudioType::Tone { tone: ToneType::Square, pitch: 440.0, duration: 2.0 })
        .build()
        .unwrap();

    assert_eq!(audio.get_volume().unwrap(), 1.0);
    assert_eq!(audio.get_loop().unwrap(), 0);
    assert_eq!(audio.get_duration().unwrap(), 2000);
    audio.get_remaining().unwrap();
    audio.get_start_time().unwrap();
    audio.get_end_time().unwrap();
    audio.is_paused().unwrap();

    assert_eq!(replit_audio::is_disabled().unwrap(), false);
    assert_eq!(replit_audio::is_running().unwrap(), true);

    thread::sleep(Duration::from_secs(1));

    audio.update(&AudioUpdate { volume: 0.1, paused: false, does_loop: false, loop_count: -1 }).unwrap();
}
