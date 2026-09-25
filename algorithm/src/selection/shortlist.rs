//! 候補の絞り込み(約7曲)
//!
//! scoring.rs(評価軸ベースのtotal_score)とhistory.rs(履歴補正)を
//! 組み合わせたfinal_scoreでソートし、上位N件を返す。
//! 「約7曲と完全に同義である必要はない」という設計意図に対し、
//! 現時点ではシンプルに上位N件を返す実装から始める。

use crate::data::track::Track;
use crate::selection::flow::big_wave::BigWave;
use crate::selection::history::{final_score, history_bonus};
use crate::selection::scoring::{total_score, CandidateEvaluation};

/// final_score(total_score + history_bonus)でソートし、上位count件のTrackを返す。
pub fn shortlist<'a>(
    evaluations: &[CandidateEvaluation<'a>],
    big_wave: &BigWave,
    energy_importance: f32,
    bpm_importance: f32,
    brightness_importance: f32,
    key_importance: f32,
    count: usize,
) -> Vec<&'a Track> {
    let mut scored: Vec<(&'a Track, f32)> = evaluations
        .iter()
        .map(|eval| {
            let ts = total_score(eval, energy_importance, bpm_importance, brightness_importance, key_importance);
            let hb = history_bonus(big_wave, &eval.track.id);
            (eval.track, final_score(ts, hb))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().take(count).map(|(track, _)| track).collect()
}