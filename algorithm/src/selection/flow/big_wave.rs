//! 49曲周期での流れ(Big Wave)

use std::collections::VecDeque;

use crate::config::BIG_WAVE_SIZE;
use crate::data::track::TrackId;

pub struct BigWave {
    recent_tracks: VecDeque<TrackId>,
    /// 周期的な位置(0〜BIG_WAVE_SIZE-1)。EnergyCurveの目標値を参照する軸として使う。
    position: usize,
}

impl BigWave {
    pub fn new() -> Self {
        BigWave {
            recent_tracks: VecDeque::with_capacity(BIG_WAVE_SIZE),
            position: 0,
        }
    }

    /// 新しく選曲された曲を追加する。BIG_WAVE_SIZEを超えたら最も古いものを捨てる。
    /// 位置カウンタも進める(BIG_WAVE_SIZEに達したら0に戻る)。
    pub fn push(&mut self, track_id: TrackId) {
        if self.recent_tracks.len() == BIG_WAVE_SIZE {
            self.recent_tracks.pop_front();
        }
        self.recent_tracks.push_back(track_id);
        self.position = (self.position + 1) % BIG_WAVE_SIZE;
    }

    pub fn tracks(&self) -> &VecDeque<TrackId> {
        &self.recent_tracks
    }

    pub fn current_position(&self) -> usize {
        self.position
    }
}