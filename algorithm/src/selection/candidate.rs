//! 曲候補
//!
//! 全曲(Library)から、選択対象になり得る候補集合を作る。
//! この段階では「良い曲」の判断は行わない(スコアリングはPhase6-5)。
//! 明確に除外すべきもの(現在再生中の曲自身)のみを除く。

use crate::data::library::Library;
use crate::data::track::Track;

pub fn generate_candidates<'a>(
    library: &'a Library,
    current_track_id: Option<&str>,
) -> Vec<&'a Track> {
    library
        .tracks()
        .iter()
        .filter(|track| match current_track_id {
            Some(id) => track.id != id,
            None => true,
        })
        .collect()
}