use crate::audio::mono_audio::MonoAudio;
use crate::analyzer::features::RmsFeature;
use crate::analyzer::framing;
use crate::config::{FRAME_SIZE, HOP_SIZE};

pub fn calculate_rms(audio: &MonoAudio) -> RmsFeature {
    let frames = framing::extract_frames(&audio.samples, FRAME_SIZE, HOP_SIZE);

    let mut frame_rms = Vec::new();

    for (index, frame) in frames.iter().enumerate() {

        let sum: f32 = frame
            .iter()
            .map(|sample| sample * sample)
            .sum();

        let rms =
            (sum / frame.len() as f32).sqrt();

        if index % 1000 == 0 {
            println!("Frame {} RMS: {}", index, rms);
        }

        frame_rms.push(rms);
    }

    let overall_rms =
        frame_rms.iter().sum::<f32>()
        / frame_rms.len() as f32;

    RmsFeature {
        overall_rms,
        frame_rms,
    }
}