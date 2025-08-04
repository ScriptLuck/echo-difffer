// Main file
// Update 5: Implemented generic approach (removed hardcoded staff), now it may work on any PulseAudio/Pipewire device :)

mod sink;
use sink::VirtualSink;

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use libpulse_binding::def::BufferAttr;
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

fn main() {
    const APP_NAME: &str = "Echo Difffer";
    let sink_name = "EchoDifffer";

    let virtual_sink = VirtualSink::new(sink_name, APP_NAME);
    let spec = virtual_sink.original_spec();

    let buffer_attr = BufferAttr {
        maxlength: u32::MAX,
        tlength: 1024,
        prebuf: 1024,
        minreq: 256,
        fragsize: 256,
    };

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

    // spawn another thread
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    thread::spawn(move || {
        println!(
            "Type anything (+ Enter) to close the app peacefully\nOtherwise your device sound might not be broken... :("
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        r.store(false, Ordering::Relaxed);
    });

    let mut buffer = [0u8; 1024];
    while running.load(Ordering::Relaxed) {
        record.read(&mut buffer).expect("Failed to read audio");
        playback.write(&buffer).expect("Playback failed");
    }
}
