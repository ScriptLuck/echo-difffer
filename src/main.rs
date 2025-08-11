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

// Command-Line arguments
use std::env;

fn main() {
    // Constants
    const APP_NAME: &str = "Echo Difffer";
    const SINK_NAME: &str = "EchoDifffer";

    // Collect the command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if too many arguments
    if args.len() > 4 {
        println!("Please read documentation on how to use the app");
        return;
    }

    // Check if arguments are valid
    let mut envs: Vec<f64> = Vec::new();
    for (i, arg) in args.iter().skip(1).take(3).enumerate() {
        match arg.parse::<f64>() {
            Ok(a) if ((i < 2 && a >= 0.0 && a <= 1.0) || (i == 2 && a >= 1.0)) => envs.push(a),
            _ => {
                println!(
                    "Error: '{}' is not a valid number.\nPlease read documentation on how to use the app",
                    arg
                );
                return;
            }
        }
    }

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
    // Configs of volume update:

    // Desired RMS level | Set lower for quieter sounds (0.0 to 1.0)
    let target_volume_lvl = if envs.len() > 0 { envs[0] } else { 0.16 };
    // Desired gain factor | Set lower for smoother adjustments (0.0 to 1.0)
    let gain_factor = if envs.len() > 1 { envs[1] } else { 0.02 };
    // Desired clamp level | Set max virtual volume to `clamp_max`, and min to 1/`clamp_max`
    let clamp_max = if envs.len() > 2 { envs[2] } else { 2.0 };

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
        let volume_lvl = volume::calculate_volume(&buf, &spec.format);

        // Calculate target volume
        let target_volume: f64 = target_volume_lvl / volume_lvl;
        // Clamp with max & min values to avoid edge values
        let target_volume = target_volume.min(clamp_max).max(1.0 / clamp_max);

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
