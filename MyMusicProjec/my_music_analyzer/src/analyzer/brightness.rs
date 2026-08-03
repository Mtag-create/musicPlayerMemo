use crate::analyzer::features::BrightnessFeature;
use crate::analyzer::spectral::SpectralFeature;

/// spectral.centroid を固定範囲(0Hz〜ナイキスト周波数)で正規化し、
/// 曲をまたいで比較可能な明るさの指標(Brightness)を算出する。
pub fn calculate_brightness(
    spectral: &SpectralFeature,
    sample_rate: u32,
) -> BrightnessFeature {

    let nyquist = sample_rate as f32 / 2.0;

    let values =
        spectral.centroid
        .iter()
        .map(|&centroid| (centroid / nyquist).clamp(0.0, 1.0))
        .collect();

    BrightnessFeature { values }
}