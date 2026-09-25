//! summary(gzip+MessagePack、mmz_format::TrackSummary形式)の生バイト列を
//! 展開し、Trackとして渡す。実際のデコード処理はmmz_formatクレートに委譲する。

use crate::data::track::Track;
use crate::error::AlgorithmError;

/// summaryの生バイト列(JNI経由でKotlinから受け取る想定)からTrackを組み立てる。
pub fn load_track(id: String, summary_bytes: &[u8]) -> Result<Track, AlgorithmError> {
    let summary = mmz_format::load_summary_from_bytes(summary_bytes)?;
    Ok(Track::from_summary(id, summary))
}