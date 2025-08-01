// Autor: ScriptLuck

// main.rs

// Update 3: Create simple version that works (but with huge latency for some reason)

use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

fn main() {
    let spec = Spec {
        format: Format::S32le, // or whatever format you want
        channels: 2,
        rate: 44100,
    };

    let record = Simple::new(
        None,
        "Echo Difffer",
        Direction::Record,
        Some("MySink.monitor"),
        "Audio Record",
        &spec,
        None,
        None,
    )
    .expect("Failed to create simple PulseAudio connection");

    let playback = Simple::new(
        None,
        "Echo Difffer",
        Direction::Playback,
        Some("alsa_output.pci-0000_00_1f.3.analog-stereo"), // Your output
        "Audio Playback",
        &spec,
        None,
        None,
    )
    .expect("Failed to create playback stream");

    let mut buf = [0u8; 1024];
    loop {
        record.read(&mut buf).expect("Failed to read audio");
        playback.write(&buf).expect("Playback failed");
    }
}
