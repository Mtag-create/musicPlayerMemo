//! 最終決定
//!
//! shortlist(約7曲)からランダムに1曲選ぶ。
//! ランダム性はここに限定し、スコアリング自体には持ち込まない。

use rand::seq::IndexedRandom;

use crate::data::track::Track;

/// shortlistからランダムに1曲選ぶ。空の場合はNoneを返す。
pub fn select_next<'a>(shortlist: &[&'a Track]) -> Option<&'a Track> {
    let mut rng = rand::rng();
    shortlist.choose(&mut rng).copied()
}

use crate::data::library::Library;
use crate::data::track::TrackId;

/// ライブラリ全体からランダムに1曲選ぶ。
/// 「現在の曲」が存在しない(初回選曲)場合に使用する。
/// 通常の評価軸によるスコアリングは行わない。
pub fn select_random_from_library(library: &Library) -> Option<TrackId> {
    let mut rng = rand::rng();
    library.tracks().choose(&mut rng).map(|t| t.id.clone())
}