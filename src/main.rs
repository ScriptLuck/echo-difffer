// Update 2
// Found issue: the Rust programm cannot read the stream normally... 
// Using pipeware (and gpwgraph) I was able to read the audio from sink and send it to stream, 
// but reading the sink from Rust always sends 0s (nothing read)...



use libpulse_binding::{self as pulse, stream::State};
use pulse::{
    context::Context,
    mainloop::standard::Mainloop,
    sample::{Format, Spec},
    stream::{PeekResult, Stream},
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

    let stream_ptr = &capture_stream as *const _ as *mut pulse::stream::Stream;

    capture_stream.set_read_callback(Some(Box::new(move |readable_bytes| {
        unsafe {
            // Create buffer for the readable data
            match (*stream_ptr).peek() {
                Ok(PeekResult::Data(data)) => {
                    println!("Should Read :(");
                    // Process the audio data if needed
                    // Then write to playback
                    for &byte in data {
                        if byte != 0 {
                            print!("{:02x}", byte); // Print non-zero bytes
                        }
                    }
                }
                Ok(PeekResult::Hole(len)) => {
                    // Handle audio hole (silence)
                    eprintln!("Audio hole detected ({} bytes)", len);
                }
                Ok(PeekResult::Empty) => {
                    // Buffer is empty
                }
                Err(e) => {
                    eprintln!("Peek error: {}", e);
                }
            }
        }
    })));

    mainloop.run().expect("ERORR");
    Ok(())
}
