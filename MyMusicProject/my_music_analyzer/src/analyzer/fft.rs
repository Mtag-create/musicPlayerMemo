use rustfft::{
    FftPlanner,
    num_complex::Complex,
};

use crate::audio::mono_audio::MonoAudio;
use crate::config::{FRAME_SIZE, HOP_SIZE};
use crate::analyzer::features::SpectrumFeature;
use crate::analyzer::framing;

fn fft_frame(
    frame: &[f32],
    sample_rate: u32,
) -> SpectrumFeature {

    let mut buffer: Vec<Complex<f32>> =
        frame
        .iter()
        .map(|sample| {
            Complex {
                re: *sample,
                im: 0.0,
            }
        })
        .collect();


    let mut planner = FftPlanner::new();

    let fft =
        planner.plan_fft_forward(FRAME_SIZE);


    fft.process(&mut buffer);


    let magnitudes =
        buffer
        .iter()
        .take(FRAME_SIZE / 2)
        .map(|c| c.norm())
        .collect();


    let frequencies =
        (0..FRAME_SIZE / 2)
        .map(|i| {
            i as f32
            *
            sample_rate as f32
            /
            FRAME_SIZE as f32
        })
        .collect();


    SpectrumFeature {
        frequencies,
        magnitudes,
    }
}

pub fn calculate_fft_frames(
    audio: &MonoAudio,
) -> Vec<SpectrumFeature> {

    let frames =
        framing::extract_frames(&audio.samples, FRAME_SIZE, HOP_SIZE);

    frames
        .iter()
        .map(|frame| fft_frame(frame, audio.sample_rate))
        .collect()
}