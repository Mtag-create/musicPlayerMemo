use crate::analyzer::energy::EnergyFrame;
use crate::analyzer::features::{KickFeature, SpectrumFeature};
use crate::config::HOP_SIZE;

const KICK_BAND_MIN: f32 = 40.0;
const KICK_BAND_MAX: f32 = 150.0;
const RELEASE_TIME_MS: f32 = 100.0;

/// band_energy.low のフレーム間増加量(半波整流)をキック強度スコアとする(第1段階)。
///
/// onset(汎用の音の立ち上がり検出)とは役割を分離した、
/// キック(低域打撃音)専用の検出器。
fn calculate_low_diff_strength(energy: &Vec<EnergyFrame>) -> Vec<f32> {

    let mut strength = Vec::new();

    for i in 1..energy.len() {

        let low_diff =
            (energy[i].low - energy[i-1].low)
            .max(0.0);

        strength.push(low_diff);
    }

    strength
}

/// FFTスペクトルフレームから、40-150Hzの狭帯域エネルギーを抽出する(第2段階)。
fn calculate_kick_band_energy(spectra: &Vec<SpectrumFeature>) -> Vec<f32> {

    spectra
        .iter()
        .map(|spectrum| {

            let mut sum = 0.0;

            for (freq, magnitude) in spectrum
                .frequencies
                .iter()
                .zip(spectrum.magnitudes.iter())
            {

                if *freq >= KICK_BAND_MIN && *freq <= KICK_BAND_MAX {
                    sum += magnitude * magnitude;
                }
            }

            sum.sqrt()
        })
        .collect()
}

/// アタック即時・リリース時定数ベースのエンベロープフォロワー。
fn envelope_follow(values: &Vec<f32>, sample_rate: u32) -> Vec<f32> {

    let frame_interval_ms =
        (HOP_SIZE as f32 / sample_rate as f32) * 1000.0;

    let release_alpha =
        1.0 - (-frame_interval_ms / RELEASE_TIME_MS).exp();

    let mut result = Vec::with_capacity(values.len());

    let mut envelope = 0.0f32;

    for &value in values {

        if value > envelope {
            envelope = value;
        } else {
            envelope = envelope + release_alpha * (value - envelope);
        }

        result.push(envelope);
    }

    result
}

/// キック(低域打撃音)を検出する。
///
/// 第1段階: band_energy.low の増加量ベースのスコア
/// 第2段階: 40-150Hz狭帯域のエンベロープフォロワーによるスコア
pub fn analyze_kick(
    energy: &Vec<EnergyFrame>,
    spectra: &Vec<SpectrumFeature>,
    sample_rate: u32,
) -> KickFeature {

    let strength =
        calculate_low_diff_strength(energy);

    let band_energy =
        calculate_kick_band_energy(spectra);

    let bandpass_envelope =
        envelope_follow(&band_energy, sample_rate);

    KickFeature {
        strength,
        bandpass_envelope,
    }
}