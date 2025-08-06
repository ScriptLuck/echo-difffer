// Main file

// Volume Adjustments
mod volume;

// Virtual Sink
mod sink;
use sink::VirtualSink;

// I/O and sync
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// Audio control
use libpulse_binding::def::BufferAttr;
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

fn main() {
    // Constants
    const APP_NAME: &str = "Echo Difffer";
    const SINK_NAME: &str = "EchoDifffer";

    // Create Virtual Sink - Something like Sound Middleware
    let virtual_sink = VirtualSink::new(SINK_NAME, APP_NAME);

    // Get the Audio Spec from original device to create the streams accordingly
    let spec = virtual_sink.original_spec();
    assert!(spec.is_valid()); // Just make sure it is valid

    // Record buffer attributes for low-latency
    let buffer_attr: BufferAttr = BufferAttr {
        maxlength: u32::MAX,
        tlength: 1024,
        prebuf: 1024,
        minreq: 256,
        fragsize: 256,
    };

    // Record stream
    let record = Simple::new(
        None,
        APP_NAME,
        Direction::Record,
        Some(&virtual_sink.monitor_name()),
        "Audio Record",
        &spec,
        None,
        Some(&buffer_attr),
    )
    .expect("Failed to create simple PulseAudio connection");

    // Playback stream
    let playback = Simple::new(
        None,
        APP_NAME,
        Direction::Playback,
        Some(&virtual_sink.original_source()),
        "Audio Playback",
        &spec,
        None,
        None,
    )
    .expect("Failed to create playback stream");

    // Use threads and atomic bool to create a proper application loop
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    thread::spawn(move || {
        println!(
            "Type anything (+ Enter) to close the app peacefully\nOtherwise your sound might be broken... :("
        );
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Update the value to quit the application
        r.store(false, Ordering::Relaxed);
    });

    // Sound Buffer
    let mut buf = vec![0u8; 1024 * 4];

    // Update volume gradually to reduce sound corruption
    // Configs of volume update
    let target_volume_lvl = 0.2; // Desired RMS level (0.0 to 1.0)
    let gain_factor = 0.05; // Set lower for smoother adjustments (0.0 to 1.0)
    let clamp_lvl = 4.0; // Max set virtual volume to 400% (min = 1/`clamp_lvl` ~ 25%)

    let mut current_volume: f64 = 1.0;

    // Use while loop to finish the application as intended
    while running.load(Ordering::Relaxed) {
        // Read audio
        record.read(&mut buf).expect("Failed to read audio");

        // Skip the rest when full silence
        if buf.iter().all(|&b| b == 0) {
            continue;
        }

        // Calculate volume (RMS)
        let volume = volume::calculate_volume(&buf, &spec.format);

        // Calculate target volume
        let target_volume: f64 = (target_volume_lvl / volume).min(clamp_lvl).max(1.0 / clamp_lvl);
        // Gradually update virtual volume
        current_volume += gain_factor * (target_volume - current_volume);

        // Volume adjustment
        let buffer = volume::adjust_volume(&buf, &spec.format, current_volume);

        // Write/Play audio
        playback.write(&buffer).expect("Playback failed");
    }

    // Need to make sure to reach the end to trigger proper Drop of Virtual Sink
    // Otherwise it would not be removed and sound would be redirected there... :(
}
