use crate::symphonia_control::PlaybackControl;
use std::fs::File;
use rodio::{OutputStream, Sink, Source};
use rodio::buffer::SamplesBuffer;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::{get_codecs, get_probe};

pub fn play_mp3_with_symphonia(
    filename: &str,
    ctrl: PlaybackControl,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let probed = get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("No default track found")?;
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Set up rodio output
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    println!("[Symphonia] Starting playback of: {}", filename);

    // Process packets
    let mut packets = Vec::new();
    while let Ok(packet) = format.next_packet() {
        packets.push(packet);
    }

    println!("[Symphonia] Decoded {} packets", packets.len());

    let mut current_packet_idx = 0;

    while current_packet_idx < packets.len() {
        // Check stop flag
        if ctrl.is_stopped() {
            println!("[Symphonia] Stopped");
            sink.stop();
            break;
        }

        // Check pause flag
        if ctrl.is_paused() {
            println!("[Symphonia] Paused at packet {}", current_packet_idx);
            sink.pause();
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        } else {
            sink.play();
        }

        // Decode and append to sink
        if let Ok(decoded) = decoder.decode(&packets[current_packet_idx]) {
            let channels = decoded.spec().channels.count() as u16;
            let sample_rate = decoded.spec().rate;
            let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            sample_buf.copy_interleaved_ref(decoded);
            let samples = sample_buf.samples();
            let source = SamplesBuffer::new(channels, sample_rate, samples.to_vec());
            sink.append(source);
        }

        std::thread::sleep(std::time::Duration::from_millis(25));
        current_packet_idx += 1;
    }

    sink.sleep_until_end();
    println!("[Symphonia] Playback finished");
    Ok(())
}
