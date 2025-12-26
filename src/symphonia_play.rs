use crate::symphonia_control::PlaybackControl;
use std::fs::File;
use std::env;
use rodio::{OutputStream, Sink};
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
    let debug_mode = env::args().any(|arg| arg == "--debug");

    if debug_mode {
        println!("[Symphonia] Starting playback of: {}", filename);
    }

    // Get format info before starting
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;
    let mut first_packet = true;

    // Create sink and start immediately
    let sink = Sink::try_new(&stream_handle)?;
    sink.play();
    
    let mut was_paused = false;
    let mut sample_count = 0u64;

    // Stream decode and play directly
    while let Ok(packet) = format.next_packet() {
        if ctrl.is_stopped() {
            if debug_mode {
                println!("[Symphonia] Stopped at {:.1}s", sample_count as f32 / sample_rate as f32);
            }
            sink.stop();
            break;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if first_packet {
                    channels = decoded.spec().channels.count() as u16;
                    sample_rate = decoded.spec().rate;
                    if debug_mode {
                        println!("[Symphonia] Decoded @ {}Hz, {} channels", sample_rate, channels);
                    }
                    first_packet = false;
                }

                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                sample_buf.copy_interleaved_ref(decoded);
                let samples = sample_buf.samples();
                
                if samples.is_empty() {
                    continue;
                }

                // Handle pause/resume
                if ctrl.is_paused() {
                    if !was_paused {
                        if debug_mode {
                            println!("[Symphonia] Paused at {:.1}s", sample_count as f32 / sample_rate as f32);
                        }
                        sink.stop();
                        was_paused = true;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                } else if was_paused {
                    if debug_mode {
                        println!("[Symphonia] Resumed from {:.1}s", sample_count as f32 / sample_rate as f32);
                    }
                    was_paused = false;
                }

                // Append decoded chunk directly to sink
                let source = SamplesBuffer::new(channels, sample_rate, samples.to_vec());
                sink.append(source);
                sample_count += samples.len() as u64 / channels as u64;
                
                // Small sleep to prevent busy waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => continue,
        }
    }

    sink.sleep_until_end();
    if debug_mode {
        println!("[Symphonia] Playback complete");
    }
    Ok(())
}
