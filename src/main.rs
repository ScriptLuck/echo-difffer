// Create virtual sink
// > pactl load-module module-null-sink sink_name=MySink sink_properties=device.description=MySink
// But the program still does not work :(
// It looks like the sink is broken as when it is running no sound can be played at all (not just this program)

use libpulse_binding::{self as pulse, stream::State};
use pulse::{
    context::Context,
    mainloop::standard::Mainloop,
    sample::{Format, Spec},
    stream::Stream,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Audio format setup (32-bit float stereo @ 44.1kHz)
    let spec = Spec {
        format: Format::F32le,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    // PulseAudio mainloop and context
    let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
    let mut context = Context::new(&mainloop, "AudioPipe").expect("Failed to create context");
    context.connect(None, pulse::context::FlagSet::NOFLAGS, None)?;

    // Wait for context to be ready
    while context.get_state() != pulse::context::State::Ready {
        mainloop.iterate(false);
    }

    // Create capture stream (from MySink.monitor)
    let mut capture_stream = Stream::new(
        &mut context,
        "Capture",
        &spec,
        None, // Default channel map
    )
    .expect("Failed to create capture stream");
    capture_stream.connect_record(
        Some("MySink.monitor"), // Monitor your virtual sink
        None,                   // Default buffer attributes
        pulse::stream::FlagSet::NOFLAGS,
    )?;

    // Create playback stream (to default output)
    let mut playback_stream = Stream::new(&mut context, "Playback", &spec, None)
        .expect("Failed to create playback stream");
    playback_stream.connect_playback(
        None, // Default output device
        None,                                               // Default buffer attributes
        pulse::stream::FlagSet::NOFLAGS,
        None, // No volume adjustment
        None,
    )?;

    println!("Audio pipeline active. Press Ctrl+C to stop.");

    // Main processing loop
    loop {
        mainloop.iterate(false);

        // Check stream states
        if capture_stream.get_state() != State::Ready || playback_stream.get_state() != State::Ready
        {
            continue;
        }

        // Process audio chunks
        match capture_stream.peek()? {
            pulse::stream::PeekResult::Data(data) => {
                playback_stream.write(data, None, 0, pulse::stream::SeekMode::Relative)?;
            }

            pulse::stream::PeekResult::Hole(size) => {
                // Handle buffer hole (rare case)
                eprintln!("Buffer hole of {} bytes", size);
            }
            pulse::stream::PeekResult::Empty => {
                // No data available yet
            }
        }
    }
}
