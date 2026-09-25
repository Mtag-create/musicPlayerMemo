//! 候補曲の評価
//!
//! 現時点ではKey相性の関係種別を分類するところまでを実装する。
//! 分類結果をどう重み付けしてスコア化するかは、Waveの状態に応じて
//! 後段(評価軸の重要度決定と接続する段階)で行う。

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// 2つのKey間の音楽理論上の関係。
/// 優先順位: Exact > (Relative, Fifthは同格) > Parallel > Other
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRelation {
    /// 完全一致
    Exact,
    /// 平行調(調号を共有するメジャー/マイナーの組。例: D Major ⇔ B Minor)
    Relative,
    /// 完全五度関係(同じモードでトニックが完全五度上/下)
    Fifth,
    /// 同主調(トニックが同じでモードが異なる。例: D Major ⇔ D Minor)
    Parallel,
    /// 上記のいずれにも当てはまらない
    Other,
}

fn parse_key(name: &str) -> Option<(usize, bool)> {
    let mut parts = name.split_whitespace();
    let note = parts.next()?;
    let mode = parts.next()?;

    let tonic = NOTE_NAMES.iter().position(|&n| n == note)?;
    let is_major = match mode {
        "Major" => true,
        "Minor" => false,
        _ => return None,
    };

    Some((tonic, is_major))
}

/// 2つのKey名(例: "D Major")の関係を分類する。
/// 表記が想定外(パースできない)場合はOtherを返す。
pub fn classify_key_relation(a: &str, b: &str) -> KeyRelation {
    let parsed_a = parse_key(a);
    let parsed_b = parse_key(b);

    let (Some((tonic_a, major_a)), Some((tonic_b, major_b))) = (parsed_a, parsed_b) else {
        return KeyRelation::Other;
    };

    if tonic_a == tonic_b && major_a == major_b {
        return KeyRelation::Exact;
    }

    if major_a != major_b {
        let (major_tonic, minor_tonic) = if major_a {
            (tonic_a, tonic_b)
        } else {
            (tonic_b, tonic_a)
        };
        let offset = (major_tonic + 12 - minor_tonic) % 12;
        if offset == 3 {
            return KeyRelation::Relative;
        }
    }

    if tonic_a == tonic_b && major_a != major_b {
        return KeyRelation::Parallel;
    }

    if major_a == major_b {
        let diff = (tonic_a as i32 - tonic_b as i32).rem_euclid(12);
        if diff == 7 || diff == 5 {
            return KeyRelation::Fifth;
        }
    }

    KeyRelation::Other
}

use crate::data::track::Track;

/// 2つのBPM間の差分を計算する。
/// 倍/半分のテンポ関係(EDMでよくある128⇔64等)も考慮し、
/// 候補BPMを2倍・1/2倍したものとも比較して最小の差分を採用する。
pub fn bpm_difference(current_bpm: f32, candidate_bpm: f32) -> f32 {
    let candidates = [candidate_bpm, candidate_bpm * 2.0, candidate_bpm / 2.0];

    candidates
        .iter()
        .map(|&bpm| (current_bpm - bpm).abs())
        .fold(f32::INFINITY, f32::min)
}

/// 現在曲のOutroエネルギーと候補曲のIntroエネルギーの差分。
/// 値が小さいほど、曲間のエネルギーがスムーズに繋がることを意味する。
pub fn energy_transition_difference(current_outro_energy: f32, candidate_intro_energy: f32) -> f32 {
    (current_outro_energy - candidate_intro_energy).abs()
}

/// 候補曲の全体Energy平均と、EnergyCurve上の目標値との差分。
/// EnergyCurveがそのポジションの目標値を持たない場合はNoneを返す。
pub fn energy_curve_fit_difference(
    energy_curve: &crate::selection::flow::curve::EnergyCurve,
    position: usize,
    candidate_overall_energy: f32,
) -> Option<f32> {
    energy_curve
        .target_at(position)
        .map(|target| (candidate_overall_energy - target).abs())
}

/// 現在曲のOutro Brightnessと候補曲のIntro Brightnessの差分。
pub fn brightness_transition_difference(
    current_outro_brightness: f32,
    candidate_intro_brightness: f32,
) -> f32 {
    (current_outro_brightness - candidate_intro_brightness).abs()
}

pub struct CandidateEvaluation<'a> {
    pub track: &'a Track,
    pub key_relation: KeyRelation,
    pub bpm_difference: f32,
    pub energy_transition_difference: f32,
    pub energy_curve_fit_difference: Option<f32>,
    pub brightness_transition_difference: f32,
}

pub fn evaluate_candidates<'a>(
    candidates: &[&'a Track],
    current_track: &Track,
    energy_curve: &crate::selection::flow::curve::EnergyCurve,
    big_wave_position: usize,
) -> Vec<CandidateEvaluation<'a>> {
    let current_key = &current_track.key.name;
    let current_bpm = current_track.bpm;
    let current_outro_energy = current_track.energy_summary.outro;
    let current_outro_brightness = current_track.brightness_summary.outro;

    candidates
        .iter()
        .map(|&track| {
            let candidate_key = &track.key.name;
            let candidate_bpm = track.bpm;
            let candidate_intro_energy = track.energy_summary.intro;
            let candidate_intro_brightness = track.brightness_summary.intro;
            CandidateEvaluation {
                track,
                key_relation: classify_key_relation(current_key, candidate_key),
                bpm_difference: bpm_difference(current_bpm, candidate_bpm),
                energy_transition_difference: energy_transition_difference(
                    current_outro_energy,
                    candidate_intro_energy,
                ),
                energy_curve_fit_difference: energy_curve_fit_difference(
                    energy_curve,
                    big_wave_position,
                    track.energy_summary.overall,
                ),
                brightness_transition_difference: brightness_transition_difference(
                    current_outro_brightness,
                    candidate_intro_brightness,
                ),
            }
        })
        .collect()
}

/// Energy関連2軸(曲間接続・EnergyCurve適合度)を、指定された重要度で
/// 合成したスコア。値が大きいほど良い。
///
/// スコア変換式・重みの持たせ方はプロトタイプであり、最終仕様ではない。
/// energy_curve_fit_differenceがNone(EnergyCurveがそのポジションの
/// 目標値を持たない)の場合は、その項を0として扱う(スコアに寄与しない)。
pub fn energy_score(evaluation: &CandidateEvaluation, importance: f32) -> f32 {
    let transition_term = 1.0 / (1.0 + evaluation.energy_transition_difference);
    let curve_fit_term = evaluation
        .energy_curve_fit_difference
        .map(|d| 1.0 / (1.0 + d))
        .unwrap_or(0.0);

    importance * (transition_term + curve_fit_term)
}

/// BPM軸の仮スコア。重要度は固定値(検証目的の仮実装、まだトレンド分析は行わない)。
pub fn bpm_score(evaluation: &CandidateEvaluation, importance: f32) -> f32 {
    importance * (1.0 / (1.0 + evaluation.bpm_difference))
}

/// Brightness軸のスコア。重要度によって重み付けされる。
pub fn brightness_score(evaluation: &CandidateEvaluation, importance: f32) -> f32 {
    importance * (1.0 / (1.0 + evaluation.brightness_transition_difference))
}

/// Key軸のスコア。重要度によって重み付けされる。
/// key_relation_weightは「関係の良さ」をそのままスコアの土台として使う
/// (差分ではなく、大きいほど良い値のため逆数変換は不要)。
pub fn key_score(evaluation: &CandidateEvaluation, importance: f32) -> f32 {
    importance * crate::selection::flow::trend::key_relation_weight(evaluation.key_relation)
}

/// Energy軸・BPM軸・Brightness軸・Key軸を合成した合計スコア。
pub fn total_score(
    evaluation: &CandidateEvaluation,
    energy_importance: f32,
    bpm_importance: f32,
    brightness_importance: f32,
    key_importance: f32,
) -> f32 {
    energy_score(evaluation, energy_importance)
        + bpm_score(evaluation, bpm_importance)
        + brightness_score(evaluation, brightness_importance)
        + key_score(evaluation, key_importance)
}