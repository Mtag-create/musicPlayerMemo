use crate::audio::mono_audio::MonoAudio;
use crate::analyzer::features::ChromaFeature;
use crate::analyzer::framing;
use crate::config::{FRAME_SIZE, HOP_SIZE};

const NUM_OCTAVES: usize = 5;       // C2 〜 C6
const NOTES_PER_OCTAVE: usize = 12;
const TOTAL_NOTES: usize = NUM_OCTAVES * NOTES_PER_OCTAVE; // 60
const BASE_OCTAVE: i32 = 2;
const C0_FREQUENCY: f32 = 16.3516;
const Q_FACTOR: f32 = 16.817; // 1 / (2^(1/12) - 1)
const MIN_WINDOW_LENGTH: usize = 8;

/// C2〜C6、60音分の周波数を算出する。
fn note_frequencies() -> Vec<f32> {

    (0..TOTAL_NOTES)
        .map(|note_idx| {

            let octave =
                BASE_OCTAVE + (note_idx / NOTES_PER_OCTAVE) as i32;

            let semitone =
                (note_idx % NOTES_PER_OCTAVE) as f32;

            C0_FREQUENCY
                * 2f32.powi(octave)
                * 2f32.powf(semitone / 12.0)
        })
        .collect()
}

/// center を中心に length サンプルの窓を切り出す(端はクリップされる)。
fn extract_centered_window(
    samples: &[f32],
    center: usize,
    length: usize,
) -> &[f32] {

    let half = length / 2;

    let start = center.saturating_sub(half);
    let end = (start + length).min(samples.len());
    let start = start.min(samples.len());

    &samples[start..end]
}

/// Goertzelアルゴリズムで、指定周波数のエネルギー(振幅相当)を求める。
fn goertzel_magnitude(
    samples: &[f32],
    sample_rate: u32,
    frequency: f32,
) -> f32 {

    let n = samples.len();

    if n < MIN_WINDOW_LENGTH {
        return 0.0;
    }

    let window = framing::generate_hann_window(n);

    let omega =
        2.0 * std::f32::consts::PI * frequency / sample_rate as f32;

    let coeff = 2.0 * omega.cos();

    let mut s_prev = 0.0f32;
    let mut s_prev2 = 0.0f32;

    for i in 0..n {

        let x = samples[i] * window[i];

        let s = x + coeff * s_prev - s_prev2;

        s_prev2 = s_prev;
        s_prev = s;
    }

    let power =
        s_prev * s_prev
        + s_prev2 * s_prev2
        - coeff * s_prev * s_prev2;

    power.max(0.0).sqrt()
}

/// MonoAudioからCQT近似(Goertzelベース)によるChromaを計算する。
pub fn analyze_chroma(audio: &MonoAudio) -> ChromaFeature {

    let frequencies = note_frequencies();
    let sample_rate = audio.sample_rate;

    let frame_count =
        if audio.samples.len() < FRAME_SIZE {
            0
        } else {
            (audio.samples.len() - FRAME_SIZE) / HOP_SIZE + 1
        };

    let mut chroma = Vec::with_capacity(frame_count);
    let mut chroma_detailed = Vec::with_capacity(frame_count);

    for i in 0..frame_count {

        let center = i * HOP_SIZE;

        let mut detailed = [0.0f32; TOTAL_NOTES];

        for (note_idx, &freq) in frequencies.iter().enumerate() {

            let window_length =
                ((Q_FACTOR * sample_rate as f32) / freq).round() as usize;

            let window_length = window_length.max(MIN_WINDOW_LENGTH);

            let window =
                extract_centered_window(&audio.samples, center, window_length);

            detailed[note_idx] =
                goertzel_magnitude(window, sample_rate, freq);
        }

        let total: f32 = detailed.iter().sum();

        if total > 0.0 {
            for value in detailed.iter_mut() {
                *value /= total;
            }
        }

        let mut classes = [0.0f32; 12];

        for (note_idx, &value) in detailed.iter().enumerate() {
            classes[note_idx % 12] += value;
        }

        chroma_detailed.push(detailed);
        chroma.push(classes);
    }

    ChromaFeature {
        chroma,
        chroma_detailed,
    }
}