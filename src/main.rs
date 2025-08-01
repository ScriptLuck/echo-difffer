// Autor: ScriptLuck

// main.rs

// Update 4: Fixed latency issue, now all working just fine

use libpulse_binding::def::BufferAttr;
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

fn main() {
    let spec = Spec {
        format: Format::S32le,
        channels: 2,
        rate: 44100,
    };

    let buffer_attr = BufferAttr {
        maxlength: u32::MAX, // Maximum buffer length ~ auto
        tlength: 1024,       // Target length (samples)
        prebuf: 1024,        // Pre-buffering
        minreq: 256,         // Minimum request
        fragsize: 256,       // Fragment size
    };

    let record = Simple::new(
        None,
        "Echo Difffer",
        Direction::Record,
        Some("MySink.monitor"),
        "Audio Record",
        &spec,
        None,
        Some(&buffer_attr),
    )
    .expect("Failed to create simple PulseAudio connection");

    let playback = Simple::new(
        None,
        "Echo Difffer",
        Direction::Playback,
        Some("alsa_output.pci-0000_00_1f.3.analog-stereo"), // Output device
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
