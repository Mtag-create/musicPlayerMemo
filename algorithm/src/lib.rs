//! MyMusicPlayer 選曲アルゴリズムクレート「algorithm」
//!
//! Phase0時点ではモジュール構成のみを定義し、各モジュールの中身は
//! 対応するPhaseで実装する。
//!
//! - data:      Phase6-1, 6-2
//! - selection: Phase6-3〜6-7
//! - ffi:       Phase5(JNI境界、MessagePackのエンコード/デコードのみ担当)
//! - config:    Phase6以降、必要に応じて随時
//! - error:     Phase6以降、必要に応じて随時

pub mod config;
pub mod data;
pub mod error;
pub mod ffi;
pub mod selection;
pub mod state;
pub mod protocol;