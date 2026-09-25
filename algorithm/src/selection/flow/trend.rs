//! Short Wave上の履歴から、各評価軸の直近の変化傾向(トレンド)を分析するモジュール。
//!
//! 現時点ではEnergy軸のみを対象にしたプロトタイプ。
//! 判定結果を実際の重要度にどう反映するかは仮実装であり、
//! 後からBPM/Key/Brightness/EnergyCurveにも同じ構造を拡張していく想定。

use crate::config::SHORT_WAVE_SIZE;
use crate::data::library::Library;
use crate::selection::flow::short_wave::ShortWave;

/// 隣接する曲同士のEnergy差分について、「変化なし」とみなす許容幅。
/// 仮の閾値であり、最終的な数値ではない。
const ENERGY_TREND_THRESHOLD: f32 = 0.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Rising,
    Falling,
    Sustaining,
    Fluctuating,
}

/// Short Wave内の直近曲について、Energy(energy_summary.overall)の推移を分析する。
///
/// 分析不能な場合(履歴がSHORT_WAVE_SIZE未満、または履歴上のTrackIdが
/// Libraryに見つからない)はNoneを返す。推測値での補完は行わない。
///
/// 判定方法:
/// - 隣接差分の符号(正/負)が両方とも現れる場合は、合計値によらず Fluctuating とする
///   (大きく上下動して最終的に戻ってきたケースを「維持」と誤判定しないため)
/// - 符号が混在しない場合、差分の合計値で上昇/下降/維持を判定する
pub fn analyze_energy_trend(short_wave: &ShortWave, library: &Library) -> Option<Trend> {
    let track_ids = short_wave.tracks();

    if track_ids.len() < SHORT_WAVE_SIZE {
        return None;
    }

    let mut values = Vec::with_capacity(track_ids.len());
    for id in track_ids {
        let track = library.tracks().iter().find(|t| &t.id == id)?;
        values.push(track.energy_summary.overall);
    }

    let diffs: Vec<f32> = values.windows(2).map(|w| w[1] - w[0]).collect();

    let has_positive = diffs.iter().any(|&d| d > ENERGY_TREND_THRESHOLD);
    let has_negative = diffs.iter().any(|&d| d < -ENERGY_TREND_THRESHOLD);

    if has_positive && has_negative {
        return Some(Trend::Fluctuating);
    }

    let total: f32 = diffs.iter().sum();

    if total > ENERGY_TREND_THRESHOLD {
        Some(Trend::Rising)
    } else if total < -ENERGY_TREND_THRESHOLD {
        Some(Trend::Falling)
    } else {
        Some(Trend::Sustaining)
    }
}

/// トレンド判定結果から、Energy軸の重要度への影響を仮算出する。
///
/// 数値・ロジックともに仮実装のプロトタイプであり、最終仕様ではない。
/// 「明確な方向性がある(上昇/下降)→重視」「変動が激しい→重視度を下げる」
/// という大まかな考え方だけを反映した最小限の実装。
pub fn energy_axis_importance(trend: Option<Trend>) -> f32 {
    match trend {
        Some(Trend::Rising) | Some(Trend::Falling) => 1.5,
        Some(Trend::Sustaining) => 1.0,
        Some(Trend::Fluctuating) => 0.5,
        None => 1.0,
    }
}

/// 隣接するBPM差について、「変化なし」とみなす許容幅。仮の閾値。
const BPM_TREND_DIFF_THRESHOLD: f32 = 2.0;

/// 上下動の合計量がこれを超えたら「大きく変動している」とみなす。仮の閾値。
const BPM_FLUCTUATION_MAGNITUDE_THRESHOLD: f32 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpmTrend {
    /// BPMの変化が小さく安定している
    Stable,
    /// 全体としてBPMが上昇している
    Rising,
    /// 全体としてBPMが下降している
    Falling,
    /// BPMの上下動が大きい
    Fluctuating,
}

/// Short Wave内の直近曲について、BPMの推移を分析する。
///
/// 分析不能な場合(履歴不足、またはLibraryにTrackが見つからない)はNoneを返す。
///
/// 判定方法:
/// 1. 隣接差分の符号が両方(正・負)現れ、かつ差分の絶対値の合計が
///    BPM_FLUCTUATION_MAGNITUDE_THRESHOLDを超える場合は Fluctuating とする
///    (符号が混在していても、変動量自体が小さければ Fluctuating にはしない)
/// 2. それ以外は、差分の合計値で Rising / Falling / Stable を判定する
pub fn analyze_bpm_trend(short_wave: &ShortWave, library: &Library) -> Option<BpmTrend> {
    let track_ids = short_wave.tracks();

    if track_ids.len() < SHORT_WAVE_SIZE {
        return None;
    }

    let mut values = Vec::with_capacity(track_ids.len());
    for id in track_ids {
        let track = library.tracks().iter().find(|t| &t.id == id)?;
        values.push(track.bpm);
    }

    let diffs: Vec<f32> = values.windows(2).map(|w| w[1] - w[0]).collect();

    let has_positive = diffs.iter().any(|&d| d > BPM_TREND_DIFF_THRESHOLD);
    let has_negative = diffs.iter().any(|&d| d < -BPM_TREND_DIFF_THRESHOLD);
    let total_variation: f32 = diffs.iter().map(|d| d.abs()).sum();

    if has_positive && has_negative && total_variation > BPM_FLUCTUATION_MAGNITUDE_THRESHOLD {
        return Some(BpmTrend::Fluctuating);
    }

    let total: f32 = diffs.iter().sum();

    if total > BPM_TREND_DIFF_THRESHOLD {
        Some(BpmTrend::Rising)
    } else if total < -BPM_TREND_DIFF_THRESHOLD {
        Some(BpmTrend::Falling)
    } else {
        Some(BpmTrend::Stable)
    }
}

/// トレンド判定結果から、BPM軸の重要度への影響を仮算出する。
/// 数値・ロジックともに仮実装のプロトタイプであり、最終仕様ではない。
pub fn bpm_axis_importance(trend: Option<BpmTrend>) -> f32 {
    match trend {
        Some(BpmTrend::Rising) | Some(BpmTrend::Falling) => 1.5,
        Some(BpmTrend::Stable) => 1.0,
        Some(BpmTrend::Fluctuating) => 0.5,
        None => 1.0,
    }
}

/// 隣接するBrightness差分について、「変化なし」とみなす許容幅。仮の閾値。
const BRIGHTNESS_TREND_THRESHOLD: f32 = 0.02;

/// Short Wave内の直近曲について、Brightness(brightness_summary.overall)の推移を分析する。
///
/// 判定ロジックはanalyze_energy_trendと同一(符号混在チェック→合計値判定)。
/// 分析不能な場合(履歴不足、またはLibraryにTrackが見つからない)はNoneを返す。
pub fn analyze_brightness_trend(short_wave: &ShortWave, library: &Library) -> Option<Trend> {
    let track_ids = short_wave.tracks();

    if track_ids.len() < SHORT_WAVE_SIZE {
        return None;
    }

    let mut values = Vec::with_capacity(track_ids.len());
    for id in track_ids {
        let track = library.tracks().iter().find(|t| &t.id == id)?;
        values.push(track.brightness_summary.overall);
    }

    let diffs: Vec<f32> = values.windows(2).map(|w| w[1] - w[0]).collect();

    let has_positive = diffs.iter().any(|&d| d > BRIGHTNESS_TREND_THRESHOLD);
    let has_negative = diffs.iter().any(|&d| d < -BRIGHTNESS_TREND_THRESHOLD);

    if has_positive && has_negative {
        return Some(Trend::Fluctuating);
    }

    let total: f32 = diffs.iter().sum();

    if total > BRIGHTNESS_TREND_THRESHOLD {
        Some(Trend::Rising)
    } else if total < -BRIGHTNESS_TREND_THRESHOLD {
        Some(Trend::Falling)
    } else {
        Some(Trend::Sustaining)
    }
}

/// トレンド判定結果から、Brightness軸の重要度への影響を仮算出する。
/// 数値・ロジックともに仮実装のプロトタイプであり、最終仕様ではない。
pub fn brightness_axis_importance(trend: Option<Trend>) -> f32 {
    match trend {
        Some(Trend::Rising) | Some(Trend::Falling) => 1.5,
        Some(Trend::Sustaining) => 1.0,
        Some(Trend::Fluctuating) => 0.5,
        None => 1.0,
    }
}

use crate::selection::scoring::{classify_key_relation, KeyRelation};

/// KeyRelationごとの「良さ」の重み。仮の値であり最終仕様ではない。
pub fn key_relation_weight(relation: KeyRelation) -> f32 {
    match relation {
        KeyRelation::Exact => 1.0,
        KeyRelation::Relative | KeyRelation::Fifth => 0.75,
        KeyRelation::Parallel => 0.4,
        KeyRelation::Other => 0.0,
    }
}

/// Short Wave内の隣接曲同士のKey関係を評価し、平均的な「良さ」を算出する。
/// 分析不能な場合(履歴不足、またはLibraryにTrackが見つからない)はNoneを返す。
pub fn analyze_key_stability(short_wave: &ShortWave, library: &Library) -> Option<f32> {
    let track_ids = short_wave.tracks();

    if track_ids.len() < SHORT_WAVE_SIZE {
        return None;
    }

    let mut key_names = Vec::with_capacity(track_ids.len());
    for id in track_ids {
        let track = library.tracks().iter().find(|t| &t.id == id)?;
        key_names.push(track.key.name.clone());
    }

    let weights: Vec<f32> = key_names
        .windows(2)
        .map(|w| key_relation_weight(classify_key_relation(&w[0], &w[1])))
        .collect();

    Some(weights.iter().sum::<f32>() / weights.len() as f32)
}

/// Key軸の重要度を算出する。
/// 「良い関係(高い平均重み)が続いている」ほど重要度を下げ、
/// 「関係が崩れている(低い平均重み)」ほど重要度を上げる(安定性の観点、B方向)。
/// 数値・ロジックともに仮実装のプロトタイプであり、最終仕様ではない。
pub fn key_axis_importance(average_weight: Option<f32>) -> f32 {
    match average_weight {
        Some(w) => 1.5 - w,
        None => 1.0,
    }
}