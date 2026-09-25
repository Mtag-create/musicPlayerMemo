//! 全曲のTrackライブラリ。
//! 選曲アルゴリズムが対象とする全曲を保持する。
//! 履歴・Wave状態はPhase6-4で別途扱う(ここではまだ持たない)。

use crate::data::track::Track;

pub struct Library {
    tracks: Vec<Track>,
}

impl Library {
    pub fn new(tracks: Vec<Track>) -> Self {
        Library { tracks }
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }
}