//! 軽量な楽曲サマリ形式(TrackSummary)
//!
//! AnalysisOutput全体(生フレームデータ)ではなく、選曲アルゴリズムが
//! 実際に使う値だけを抽出した軽量版。MyMusicAnalyzer側で.mmzと合わせて
//! 追加出力し、MyMusicPlayer(algorithm crate)側はこちらだけを読み込む
//! ことで、転送量・メモリ使用量を大幅に削減する。

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::error::MmzError;
use crate::format::AnalysisOutput;

#[derive(Serialize, Deserialize)]
pub struct SectionSummary {
    pub start_frame: usize,
    pub end_frame: usize,
    pub section_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct KeySummary {
    pub name: String,
    pub correlation: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EnergySummary {
    pub overall: f32,
    pub intro: f32,
    pub outro: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BrightnessSummary {
    pub overall: f32,
    pub intro: f32,
    pub outro: f32,
}

#[derive(Serialize, Deserialize)]
pub struct TrackSummary {
    pub bpm: f32,
    pub key: KeySummary,
    pub energy: EnergySummary,
    pub brightness: BrightnessSummary,
    pub sections: Vec<SectionSummary>,
}

/// AnalysisOutputから軽量なTrackSummaryを計算する。
pub fn compute_summary(analysis: &AnalysisOutput) -> TrackSummary {
    let energy = compute_energy_summary(analysis);
    let brightness = compute_brightness_summary(analysis);

    let key = KeySummary {
        name: analysis.chroma.overall_key.name.clone(),
        correlation: analysis.chroma.overall_key.correlation,
    };

    let sections = analysis
        .structure
        .sections
        .iter()
        .map(|s| SectionSummary {
            start_frame: s.start_frame,
            end_frame: s.end_frame,
            section_type: s.section_type.clone(),
        })
        .collect();

    TrackSummary {
        bpm: analysis.beat.bpm,
        key,
        energy,
        brightness,
        sections,
    }
}

fn compute_energy_summary(analysis: &AnalysisOutput) -> EnergySummary {
    let values: Vec<f32> = analysis.energy_curve.frames.iter().map(|f| f.overall).collect();
    let overall = average(&values, 0, values.len());
    let intro = analysis
        .structure
        .sections
        .iter()
        .find(|s| s.section_type == "intro")
        .map(|s| average(&values, s.start_frame, s.end_frame))
        .unwrap_or(0.0);
    let outro = analysis
        .structure
        .sections
        .iter()
        .find(|s| s.section_type == "outro")
        .map(|s| average(&values, s.start_frame, s.end_frame))
        .unwrap_or(0.0);
    EnergySummary { overall, intro, outro }
}

fn compute_brightness_summary(analysis: &AnalysisOutput) -> BrightnessSummary {
    let values: Vec<f32> = analysis.brightness.frames.iter().map(|f| f.value).collect();
    let overall = average(&values, 0, values.len());
    let intro = analysis
        .structure
        .sections
        .iter()
        .find(|s| s.section_type == "intro")
        .map(|s| average(&values, s.start_frame, s.end_frame))
        .unwrap_or(0.0);
    let outro = analysis
        .structure
        .sections
        .iter()
        .find(|s| s.section_type == "outro")
        .map(|s| average(&values, s.start_frame, s.end_frame))
        .unwrap_or(0.0);
    BrightnessSummary { overall, intro, outro }
}

fn average(values: &[f32], start: usize, end: usize) -> f32 {
    let end = end.min(values.len());
    if start >= end {
        return 0.0;
    }
    let slice = &values[start..end];
    slice.iter().sum::<f32>() / slice.len() as f32
}

/// TrackSummaryをMessagePack+gzipで保存する。
pub fn save_summary(summary: &TrackSummary, path: &Path) -> Result<(), MmzError> {
    let msgpack_bytes =
        rmp_serde::to_vec(summary).map_err(|e| MmzError::Serialization(e.to_string()))?;
    let file = fs::File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&msgpack_bytes)?;
    encoder.finish()?;
    Ok(())
}

/// gzip圧縮されたMessagePackファイルからTrackSummaryを読み込む。
pub fn load_summary(path: &Path) -> Result<TrackSummary, MmzError> {
    let file = fs::File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    rmp_serde::from_slice(&bytes).map_err(|e| MmzError::Serialization(e.to_string()))
}

/// メモリ上のバイト列からTrackSummaryを復元する(JNI経由での利用を想定)。
pub fn load_summary_from_bytes(bytes: &[u8]) -> Result<TrackSummary, MmzError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    rmp_serde::from_slice(&decompressed).map_err(|e| MmzError::Serialization(e.to_string()))
}