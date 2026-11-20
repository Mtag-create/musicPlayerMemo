//! 選曲履歴による補正
//!
//! total_score(Energy/BPM/Brightness/Keyの評価軸スコア)とは責務を分離し、
//! 直近に選ばれた曲ほど選ばれにくくなるペナルティ(0以下の値)を計算する。
//! Big Wave(直近49曲の履歴)を参照元とする。
//!
//! 数値・減衰方法ともに仮実装のプロトタイプであり、最終仕様ではない。

use crate::data::track::TrackId;
use crate::selection::flow::big_wave::BigWave;

/// 直前(distance=1)のペナルティ最大値。仮の値。
const MAX_HISTORY_PENALTY: f32 = 10.0;

/// 減衰率(0〜1)。値が小さいほど急速に0へ近づく。仮の値。
const HISTORY_DECAY_RATE: f32 = 0.98;

/// Big Wave上でtrack_idが「末尾(直前)から何番目か」を距離として求め、
/// 指数関数的に減衰するペナルティ(0以下)を返す。
/// 履歴に見つからない場合(未再生、またはBig Waveの範囲より前)は0.0(補正なし)。
pub fn history_bonus(big_wave: &BigWave, track_id: &TrackId) -> f32 {
    let distance = big_wave
        .tracks()
        .iter()
        .rev()
        .position(|id| id == track_id)
        .map(|i| i + 1); // 末尾(直前に選ばれた曲)がdistance=1

    match distance {
        Some(d) => -(MAX_HISTORY_PENALTY * HISTORY_DECAY_RATE.powi(d as i32 - 1) - 1.0).max(0.0),
        None => 0.0,
    }
}

/// total_scoreとhistory_bonusを合成した最終スコア。
pub fn final_score(total_score: f32, history_bonus: f32) -> f32 {
    total_score + history_bonus
}