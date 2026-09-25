//! 次の曲を選ぶ核心モジュール
//!
//! 実装順序(Phase6-3〜6-7): candidate → flow → scoring → probability → selector

pub mod candidate;
pub mod flow;
pub mod history;
pub mod probability;
pub mod scoring;
pub mod selector;
pub mod shortlist;