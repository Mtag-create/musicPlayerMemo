//! アプリ全体で共有する選曲状態(プロセス内に1つだけ)

use std::sync::{Mutex, OnceLock};

use crate::data::library::Library;
use crate::data::track::TrackId;
use crate::selection::flow::big_wave::BigWave;
use crate::selection::flow::curve::EnergyCurve;
use crate::selection::flow::short_wave::ShortWave;

pub struct AppState {
    pub library: Library,
    pub short_wave: ShortWave,
    pub big_wave: BigWave,
    pub energy_curve: EnergyCurve,
    pub current_track_id: Option<TrackId>,
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

/// アプリ起動時に一度だけ呼び出し、状態を初期化する。
pub fn init(library: Library) -> bool {
    let state = AppState {
        library,
        short_wave: ShortWave::new(),
        big_wave: BigWave::new(),
        energy_curve: EnergyCurve::new_flat(0.5),
        current_track_id: None,
    };
    STATE.set(Mutex::new(state)).is_ok()
}

/// 状態にアクセスする。init()が未実行の場合はNoneを返す。
pub fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AppState) -> R,
{
    STATE.get().map(|m| {
        let mut guard = m.lock().unwrap();
        f(&mut guard)
    })
}