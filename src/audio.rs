use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::*, HeapRb};

pub fn setup() -> ringbuf::HeapCons<f32> {
    // This could use further streamlining in order to get rid of manually selecting the device in
    // pavucontrol
    std::env::set_var("PIPEWIRE_NODE", "{ stream.capture.sink=true }");

    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("No default input device found");

    let config = device
        .default_input_config()
        .expect("Failed to get default config");

    let rb = HeapRb::<f32>::new(8192);
    let (mut prod, cons) = rb.split();

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                // HEARTBEAT DEBUG: Print a dot if we get non-zero data
                let mut has_sound = false;
                for &sample in data {
                    if sample.abs() > 0.001 {
                        has_sound = true;
                    }
                    let _ = prod.try_push(sample);
                }
                if has_sound {
                    // This will spam your console, but it proves audio is coming in!
                    // Remove once you see it working.
                    // print!(".");
                }
            },
            |err| eprintln!("[ERROR] Stream error: {}", err),
            None,
        )
        .expect("Failed to build stream");

    stream.play().expect("Failed to start stream");
    Box::leak(Box::new(stream));

    cons
}
