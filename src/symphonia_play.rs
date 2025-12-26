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

    // Decode entire MP3 into samples
    let mut all_samples = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;

    while let Ok(packet) = format.next_packet() {
        match decoder.decode(&packet) {
            Ok(decoded) => {
                channels = decoded.spec().channels.count() as u16;
                sample_rate = decoded.spec().rate;
                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                sample_buf.copy_interleaved_ref(decoded);
                all_samples.extend_from_slice(sample_buf.samples());
            }
            Err(_) => continue,
        }
    }

    if debug_mode {
        println!("[Symphonia] Buffered {} samples @ {}Hz, {} channels", all_samples.len(), sample_rate, channels);
    }

    if all_samples.is_empty() {
        if debug_mode {
            println!("[Symphonia] No samples decoded!");
        }
        return Ok(());
    }

    // Play using chunks with pause control
    let chunk_size = (sample_rate as usize / 20) * channels as usize; // 50ms chunks
    let mut position = 0;
    let total_samples = all_samples.len();
    let mut was_paused = false;
    let sink = Sink::try_new(&stream_handle)?;
    sink.play();

    if debug_mode {
        println!("[Symphonia] Chunk size: {} samples ({:.0}ms)", chunk_size, (chunk_size as f32 / sample_rate as f32 / channels as f32) * 1000.0);
    }

    while position < total_samples {
        if ctrl.is_stopped() {
            if debug_mode {
                println!("[Symphonia] Stopped at position {}", position);
            }
            sink.stop();
            break;
        }

        if ctrl.is_paused() {
            if !was_paused {
                if debug_mode {
                    println!("[Symphonia] Paused at {:.1}s (clearing sink buffer)", position as f32 / (sample_rate as f32 * channels as f32));
                }
                // Clear the sink to stop immediately
                sink.stop();
                was_paused = true;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        } else {
            if was_paused {
                if debug_mode {
                    println!("[Symphonia] Resumed from {:.1}s (restarting sink)", position as f32 / (sample_rate as f32 * channels as f32));
                }
                // Create new sink on resume
                was_paused = false;
            }
            
            // Only append chunk when not paused
            let end = std::cmp::min(position + chunk_size, total_samples);
            let chunk = &all_samples[position..end];
            
            if !chunk.is_empty() {
                let source = SamplesBuffer::new(channels, sample_rate, chunk.to_vec());
                sink.append(source);
                sink.play();
            }

            position = end;
        }
        
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

    sink.sleep_until_end();
    if debug_mode {
        println!("[Symphonia] Playback complete");
    }
    Ok(())
}
