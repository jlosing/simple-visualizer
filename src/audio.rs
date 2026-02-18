use pipewire as pw;
use pw::spa::param::audio::AudioInfoRaw;
use pw::spa::pod::Pod;
use ringbuf::{traits::*, HeapRb};
use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
struct StreamData {
    channels: AtomicU32,
    format: std::sync::Mutex<AudioInfoRaw>,
}

pub fn setup() -> ringbuf::HeapCons<f32> {
    let rb = HeapRb::<f32>::new(8192);
    let (mut prod, cons) = rb.split();

    std::thread::spawn(move || {
        let mainloop =
            pw::main_loop::MainLoopBox::new(None).expect("Failed to create PipeWire mainloop");
        let context =
            pw::context::ContextBox::new(mainloop.loop_(), None).expect("Failed to create context");
        let core = context
            .connect(None)
            .expect("Failed to connect to PipeWire core");

        let props = pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Music",
            *pw::keys::STREAM_CAPTURE_SINK => "true",
        };

        let stream = pw::stream::StreamBox::new(&core, "rust-visualizer-capture", props)
            .expect("Failed to create stream");

        let _listener = stream
            .add_local_listener_with_user_data(StreamData::default())
            .param_changed(|_, user_data, id, param| {
                let Some(param) = param else { return };
                if id != pw::spa::param::ParamType::Format.as_raw() {
                    return;
                }
                if let Ok(mut format) = user_data.format.lock() {
                    if format.parse(param).is_ok() {
                        user_data
                            .channels
                            .store(format.channels(), Ordering::Relaxed);
                    }
                }
            })
            .process(move |stream, user_data| {
                let Some(mut buffer) = stream.dequeue_buffer() else {
                    return;
                };
                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }

                let data = &mut datas[0];
                let n_channels = user_data.channels.load(Ordering::Relaxed).max(1) as usize;

                let chunk = data.chunk();
                let n_bytes = chunk.size() as usize;

                if n_bytes == 0 {
                    return;
                }

                if let Some(samples_bytes) = data.data() {
                    let valid_bytes = &samples_bytes[..n_bytes.min(samples_bytes.len())];

                    let mut i = 0;
                    for chunk in valid_bytes.chunks_exact(mem::size_of::<f32>()) {
                        // Simple Mono Mixdown: taking every 1st sample of a stereo frame.
                        // This is ideal for FFT visualizers to prevent phase cancellation.
                        if i % n_channels == 0 {
                            let sample = f32::from_le_bytes(chunk.try_into().unwrap_or([0; 4]));
                            let _ = prod.try_push(sample);
                        }
                        i += 1;
                    }
                }
            })
            .register()
            .expect("Failed to register listener");

        let mut audio_info = AudioInfoRaw::new();
        audio_info.set_format(pw::spa::param::audio::AudioFormat::F32LE);
        audio_info.set_rate(48000);
        audio_info.set_channels(2);

        let obj = pw::spa::pod::Object {
            type_: pw::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: pw::spa::param::ParamType::EnumFormat.as_raw(),
            properties: audio_info.into(),
        };

        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
        .expect("Failed to serialize format")
        .0
        .into_inner();

        let mut params = [Pod::from_bytes(&values).unwrap()];

        stream
            .connect(
                pw::spa::utils::Direction::Input,
                None,
                pw::stream::StreamFlags::AUTOCONNECT
                    | pw::stream::StreamFlags::MAP_BUFFERS
                    | pw::stream::StreamFlags::RT_PROCESS,
                &mut params,
            )
            .expect("Failed to connect stream");

        mainloop.run();
    });

    cons
}
