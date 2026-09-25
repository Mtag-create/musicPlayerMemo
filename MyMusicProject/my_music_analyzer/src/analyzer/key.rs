use crate::analyzer::features::{
    KeyCandidate, KeyEstimate, KeyFeature, SectionFeature, SectionKey,
};

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F",
    "F#", "G", "G#", "A", "A#", "B",
];

// Krumhansl-Kessler 標準プロファイル(トニックを0番目とした相対重み)
const MAJOR_PROFILE: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09,
    2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];

const MINOR_PROFILE: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53,
    2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

/// base_profile を tonic だけ回転させ、絶対ピッチクラス基準のプロファイルにする。
fn rotate_profile(base_profile: &[f32; 12], tonic: usize) -> [f32; 12] {

    let mut result = [0.0f32; 12];

    for i in 0..12 {
        result[(i + tonic) % 12] = base_profile[i];
    }

    result
}

/// 2つの12次元ベクトルのピアソン相関係数を計算する。
fn pearson_correlation(a: &[f32; 12], b: &[f32; 12]) -> f32 {

    let mean_a: f32 = a.iter().sum::<f32>() / 12.0;
    let mean_b: f32 = b.iter().sum::<f32>() / 12.0;

    let mut numerator = 0.0;
    let mut denom_a = 0.0;
    let mut denom_b = 0.0;

    for i in 0..12 {

        let da = a[i] - mean_a;
        let db = b[i] - mean_b;

        numerator += da * db;
        denom_a += da * da;
        denom_b += db * db;
    }

    if denom_a == 0.0 || denom_b == 0.0 {
        return 0.0;
    }

    numerator / (denom_a.sqrt() * denom_b.sqrt())
}

/// 指定範囲([start, end])のChromaフレームを平均する。
fn average_chroma(chroma: &Vec<[f32; 12]>, start: usize, end: usize) -> [f32; 12] {

    let mut sum = [0.0f32; 12];
    let mut count = 0;

    let end = end.min(chroma.len().saturating_sub(1));

    for i in start..=end {

        for j in 0..12 {
            sum[j] += chroma[i][j];
        }

        count += 1;
    }

    if count > 0 {
        for value in sum.iter_mut() {
            *value /= count as f32;
        }
    }

    sum
}

/// 与えられたChromaベクトルから、上位3候補のキー推定を行う。
fn estimate_key(chroma_vector: &[f32; 12]) -> KeyEstimate {

    let mut candidates = Vec::new();

    for tonic in 0..12 {

        let major = rotate_profile(&MAJOR_PROFILE, tonic);
        let minor = rotate_profile(&MINOR_PROFILE, tonic);

        let major_corr = pearson_correlation(chroma_vector, &major);
        let minor_corr = pearson_correlation(chroma_vector, &minor);

        candidates.push(KeyCandidate {
            name: format!("{} Major", NOTE_NAMES[tonic]),
            correlation: major_corr,
        });

        candidates.push(KeyCandidate {
            name: format!("{} Minor", NOTE_NAMES[tonic]),
            correlation: minor_corr,
        });
    }

    candidates.sort_by(|a, b| b.correlation.partial_cmp(&a.correlation).unwrap());
    candidates.truncate(3);

    KeyEstimate { candidates }
}

/// Chroma(フレーム列)とセクション分割結果から、曲全体・セクションごとのキーを推定する。
pub fn analyze_key(
    chroma: &Vec<[f32; 12]>,
    sections: &Vec<SectionFeature>,
) -> KeyFeature {

    let overall_chroma =
        average_chroma(chroma, 0, chroma.len().saturating_sub(1));

    let overall = estimate_key(&overall_chroma);

    let mut section_keys = Vec::new();

    for section in sections {

        let section_chroma =
            average_chroma(chroma, section.start, section.end);

        let estimate = estimate_key(&section_chroma);

        section_keys.push(SectionKey {
            label: section.label.clone(),
            start: section.start,
            end: section.end,
            estimate,
        });
    }

    KeyFeature {
        overall,
        sections: section_keys,
    }
}