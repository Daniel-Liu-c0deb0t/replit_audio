# replit_audio
Rust library for playing audio in [repl.it](https://repl.it/).

## Usage
Add
```
replit_audio = "0.1.0"
```
to your `Cargo.toml` file. This crate is available on [crates.io](https://crates.io/crates/replit_audio).

Documentation is available on [docs.rs](https://docs.rs/replit_audio).

You can view an example on [repl.it](https://repl.it/@daniel_school/replitaudiodemo).
Tests should be ran on [repl.it](https://repl.it/).

To play an audio file, create an `Audio` instance using the `AudioBuilder`:
```Rust
let audio = AudioBuilder::new(&AudioType::File { file: FileType::Wav, path: "audio.wav".to_string() })
    .volume(1.0)
    .does_loop(true)
    .loop_count(-1)
    .build()
    .unwrap();
```
Then, you can obtain certain properties of the audio you played:
```Rust
audio.get_duration().unwrap();
audio.get_remaining().unwrap();
audio.get_start_time().unwrap();
audio.get_end_time().unwrap();
audio.is_paused().unwrap();
// etc.
```
You can also play a tone:
```Rust
let mut audio = AudioBuilder::new(&AudioType::Tone { tone: ToneType::Square, pitch: 440.0, duration: 2.0 })
    .build()
    .unwrap();
```
It is possible to update a playing audio instance:
```Rust
audio.update(&AudioUpdate { volume: 0.1, paused: false, does_loop: false, loop_count: -1 }).unwrap();
```

## License
[MIT](LICENSE)
