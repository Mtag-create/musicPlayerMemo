//! .mmz(gzip圧縮されたMessagePack)のデータ形式定義。
//! MyMusicAnalyzerとalgorithmの両方がこの定義を共有する。

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::error::MmzError;

#[derive(Serialize, Deserialize)]
pub struct FramePositionOutput {
    pub frame: usize,
    pub time_seconds: f32,
}

#[derive(Serialize, Deserialize)]
pub struct FrameValueOutput {
    pub frame: usize,
    pub time_seconds: f32,
    pub value: f32,
}

#[derive(Serialize, Deserialize)]
pub struct AudioOutput {
    pub sample_rate: u32,
    pub duration_seconds: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BeatOutput {
    pub bpm: f32,
    pub bpm_candidates: Vec<f32>,
    pub beats: Vec<FramePositionOutput>,
    pub beat_strength: Vec<FrameValueOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct EnergyFrameOutput {
    pub frame: usize,
    pub time_seconds: f32,
    pub rms: f32,
    pub low: f32,
    pub mid: f32,
    pub high: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EnergyOutput {
    pub overall_rms: f32,
    pub frames: Vec<EnergyFrameOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct OnsetFrameOutput {
    pub frame: usize,
    pub time_seconds: f32,
    pub strength: f32,
    pub smoothed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct OnsetOutput {
    pub frames: Vec<OnsetFrameOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct KickOutput {
    pub strength_frames: Vec<FrameValueOutput>,
    pub envelope_frames: Vec<FrameValueOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct SectionOutput {
    pub start_frame: usize,
    pub start_time_seconds: f32,
    pub end_frame: usize,
    pub end_time_seconds: f32,
    pub section_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct StructureOutput {
    pub sections: Vec<SectionOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct EnergyCurveFrameOutput {
    pub frame: usize,
    pub time_seconds: f32,
    pub overall: f32,
    pub low: f32,
    pub mid: f32,
    pub high: f32,
    pub delta_overall: f32,
    pub delta_low: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EnergyCurveOutput {
    pub frames: Vec<EnergyCurveFrameOutput>,
    pub peaks: Vec<FramePositionOutput>,
    pub valleys: Vec<FramePositionOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct BrightnessOutput {
    pub frames: Vec<FrameValueOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct ChromaFrameOutput {
    pub frame: usize,
    pub time_seconds: f32,
    pub values: [f32; 12],
}

#[derive(Serialize, Deserialize)]
pub struct KeyCandidateOutput {
    pub name: String,
    pub correlation: f32,
}

#[derive(Serialize, Deserialize)]
pub struct KeyEstimateOutput {
    pub name: String,
    pub correlation: f32,
    pub candidates: Vec<KeyCandidateOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct SectionKeyOutput {
    pub start_frame: usize,
    pub end_frame: usize,
    pub key: KeyEstimateOutput,
}

#[derive(Serialize, Deserialize)]
pub struct ChromaOutput {
    pub frames: Vec<ChromaFrameOutput>,
    pub overall_key: KeyEstimateOutput,
    pub section_keys: Vec<SectionKeyOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub version: u32,
    pub audio: AudioOutput,
    pub beat: BeatOutput,
    pub energy: EnergyOutput,
    pub energy_curve: EnergyCurveOutput,
    pub onset: OnsetOutput,
    pub kick: KickOutput,
    pub structure: StructureOutput,
    pub chroma: ChromaOutput,
    pub brightness: BrightnessOutput,
}

/// AnalysisOutputをMessagePackにシリアライズし、gzip圧縮してファイルに保存する。
pub fn save_binary(output: &AnalysisOutput, path: &Path) -> Result<(), MmzError> {
    let msgpack_bytes =
        rmp_serde::to_vec(output).map_err(|e| MmzError::Serialization(e.to_string()))?;

    let file = fs::File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&msgpack_bytes)?;
    encoder.finish()?;

    Ok(())
}

/// gzip圧縮されたMessagePackファイルを読み込み、AnalysisOutputに復元する。
pub fn load_binary(path: &Path) -> Result<AnalysisOutput, MmzError> {
    let file = fs::File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    rmp_serde::from_slice(&bytes).map_err(|e| MmzError::Serialization(e.to_string()))
}

/// メモリ上のバイト列(gzip圧縮されたMessagePack)からAnalysisOutputを復元する。
/// JNI経由でファイルではなくバイト列として.mmzを受け取るケース(algorithm側)で使用する。
pub fn load_from_bytes(bytes: &[u8]) -> Result<AnalysisOutput, MmzError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    rmp_serde::from_slice(&decompressed).map_err(|e| MmzError::Serialization(e.to_string()))
}