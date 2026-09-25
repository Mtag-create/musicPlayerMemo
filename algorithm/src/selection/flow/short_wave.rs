//! 直近7曲の中での流れ(Short Wave)
//!
//! 現段階では履歴を保持するだけの入れ物。
//! Energy/BPM等の変化を分析する機能は、スコアリングと接続する段階で追加する。

use std::collections::VecDeque;

use crate::config::SHORT_WAVE_SIZE;
use crate::data::track::TrackId;

pub struct ShortWave {
    recent_tracks: VecDeque<TrackId>,
}

impl ShortWave {
    pub fn new() -> Self {
        ShortWave {
            recent_tracks: VecDeque::with_capacity(SHORT_WAVE_SIZE),
        }
    }

    /// 新しく選曲された曲を追加する。SHORT_WAVE_SIZEを超えたら最も古いものを捨てる。
    pub fn push(&mut self, track_id: TrackId) {
        if self.recent_tracks.len() == SHORT_WAVE_SIZE {
            self.recent_tracks.pop_front();
        }
        self.recent_tracks.push_back(track_id);
    }

    pub fn tracks(&self) -> &VecDeque<TrackId> {
        &self.recent_tracks
    }
}