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
    const RECORD_BUFFER_ATTR: BufferAttr = BufferAttr {
        maxlength: u32::MAX,
        tlength: 1024,
        prebuf: 1024,
        minreq: 256,
        fragsize: 256,
    };

    // Create Virtual Sink - Something like Sound Middleware
    let virtual_sink = VirtualSink::new(SINK_NAME, APP_NAME);

    // Get the Audio Spec from original device to create the streams accordingly
    let spec = virtual_sink.original_spec();
    assert!(spec.is_valid()); // Just make sure it is valid

    // Record stream
    let record = Simple::new(
        None,
        APP_NAME,
        Direction::Record,
        Some(&virtual_sink.monitor_name()),
        "Audio Record",
        &spec,
        None,
        Some(&RECORD_BUFFER_ATTR),
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
            "Type anything (+ Enter) to close the app peacefully\nOtherwise your device sound might not be broken... :("
        );
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Update the value to quit the application
        r.store(false, Ordering::Relaxed);
    });

    // Sound Buffer
    let mut buf = vec![0u8; 1024]; // Raw bytes (format-agnostic)

    // Use while loop to finish the application as intended
    while running.load(Ordering::Relaxed) {
        // Read audio
        record.read(&mut buf).expect("Failed to read audio");

        // Calculate volume (RMS)
        let volume = volume::calculate_volume(&buf, &spec.format);

        // Skip the rest when no sound detected
        if volume == 0 as f64 {
            continue;
        }

        // Apply volume adjustment
        let target_volume = 0.5; // Desired RMS level (0.0 to 1.0)
        let gain = (target_volume / volume).min(2.0); // Clamp gain to 2x
        let buffer = volume::adjust_volume(&buf, &spec.format, gain);

        // Write/Play audio
        playback.write(&buffer).expect("Playback failed");
    }

    // Need to make sure to reach the end to trigger proper Drop of Virtual Sink
    // Otherwise it would not be removed and sound would be redirected there... :(
}
