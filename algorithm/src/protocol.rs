//! Kotlin↔Rust間のJNIリクエスト/レスポンス型定義。
//!
//! mmz_format(MyMusicAnalyzerとの共有データ形式)とは別に、
//! MyMusicPlayer独自のJNI通信用として新設する。
//! マップ形式(フィールド名付き)でシリアライズし、
//! フィールド追加・順序変更に強い形にする。

use serde::{Deserialize, Serialize};

/// ライブラリ初期化: 1曲分のエントリ(ID + .mmzの生バイト列)
#[derive(Serialize, Deserialize)]
pub struct MmzFileEntry {
    pub id: String,
    pub mmz_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct InitLibraryRequest {
    pub files: Vec<MmzFileEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct InitLibraryResponse {
    pub success: bool,
    pub track_count: usize,
    /// 読み込みに失敗した曲のID一覧(部分的な失敗を許容する場合に使用)
    pub failed_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SelectNextRequest {
    pub energy_multiplier: f32,
    pub bpm_multiplier: f32,
    pub key_multiplier: f32,
    pub brightness_multiplier: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SelectNextResponse {
    pub next_track_id: Option<String>,
    /// 選ばれた曲のEnergy(overall)。next_track_idがNoneの場合はNone。
    pub energy: Option<f32>,
    /// 選ばれた曲のBPM。
    pub bpm: Option<f32>,
    /// 選ばれた曲のKey名("C Major"等)。
    pub key_name: Option<String>,
    /// Key検出の相関値。
    pub key_correlation: Option<f32>,
    /// 選ばれた曲のBrightness(overall)。
    pub brightness: Option<f32>,
    /// この選曲で実際に使われたEnergy軸の重要度(トレンド由来の係数×ユーザー重み)。
    /// 初回選曲(current_trackが無くランダム選出)の場合はNone。
    pub energy_importance: Option<f32>,
    /// この選曲で実際に使われたBPM軸の重要度。
    pub bpm_importance: Option<f32>,
    /// この選曲で実際に使われたBrightness軸の重要度。
    pub brightness_importance: Option<f32>,
    /// この選曲で実際に使われたKey軸の重要度。
    pub key_importance: Option<f32>,
}

/// 曲一覧画面などから、ユーザーが手動で選んだ曲を「現在の曲」として設定するリクエスト。
/// selectNextとは違いスコアリングは行わず、指定されたtrack_idをそのまま反映する。
#[derive(Serialize, Deserialize)]
pub struct SetCurrentTrackRequest {
    pub track_id: String,
}