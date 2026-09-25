use crate::audio::mono_audio::MonoAudio;
use mmz_format::{
    AnalysisOutput, AudioOutput, BeatOutput, BrightnessOutput, ChromaFrameOutput, ChromaOutput,
    EnergyCurveFrameOutput, EnergyCurveOutput, EnergyFrameOutput, EnergyOutput,
    FramePositionOutput, FrameValueOutput, KeyCandidateOutput, KeyEstimateOutput, KickOutput,
    OnsetFrameOutput, OnsetOutput, SectionKeyOutput, SectionOutput, StructureOutput,
};
pub use mmz_format::{load_binary, save_binary};
use crate::analyzer::energy::EnergyFrame;
use crate::analyzer::features::{
    BeatFeature, BrightnessFeature, ChromaFeature, EnergyCurveFeature,
    KeyFeature, KickFeature, OnsetFeature, RmsFeature,
};
use crate::config::HOP_SIZE;

const FORMAT_VERSION: u32 = 1;

fn frame_time(frame: usize, sample_rate: u32) -> f32 {
    (frame as f32 * HOP_SIZE as f32) / sample_rate as f32
}

fn section_type_label(label: &str) -> &'static str {
    match label {
        "イントロ" => "intro",
        "ビルドアップ" => "buildup",
        "ドロップ" => "drop",
        "ブレイクダウン" => "breakdown",
        "維持" => "sustain",
        "アウトロ" => "outro",
        _ => "unknown",
    }
}

fn key_estimate_output(
    candidates: &Vec<crate::analyzer::features::KeyCandidate>,
) -> KeyEstimateOutput {

    let best = &candidates[0];

    let candidate_outputs =
        candidates
        .iter()
        .map(|c| KeyCandidateOutput {
            name: c.name.clone(),
            correlation: c.correlation,
        })
        .collect();

    KeyEstimateOutput {
        name: best.name.clone(),
        correlation: best.correlation,
        candidates: candidate_outputs,
    }
}

/// 各解析結果を、出力用の AnalysisOutput にまとめる。
pub fn build_output(
    audio: &MonoAudio,
    rms: &RmsFeature,
    energy_frames: &Vec<EnergyFrame>,
    onset: &OnsetFeature,
    beat: &BeatFeature,
    kick: &KickFeature,
    energy_curve: &EnergyCurveFeature,
    chroma: &ChromaFeature,
    key: &KeyFeature,
    brightness: &BrightnessFeature,
) -> AnalysisOutput {

    let sample_rate = audio.sample_rate;

    let audio_output = AudioOutput {
        sample_rate,
        duration_seconds:
            audio.samples.len() as f32 / sample_rate as f32,
    };

    let beat_output = BeatOutput {
        bpm: beat.bpm,
        bpm_candidates: beat.bpm_candidates.clone(),
        beats:
            beat.beats
            .iter()
            .map(|&frame| FramePositionOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
            })
            .collect(),
        beat_strength:
            beat.beat_strength
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let frame = index + 1;
                FrameValueOutput {
                    frame,
                    time_seconds: frame_time(frame, sample_rate),
                    value,
                }
            })
            .collect(),
    };

    let energy_output = EnergyOutput {
        overall_rms: rms.overall_rms,
        frames:
            energy_frames
            .iter()
            .map(|f| EnergyFrameOutput {
                frame: f.index,
                time_seconds: frame_time(f.index, sample_rate),
                rms: f.rms,
                low: f.low,
                mid: f.mid,
                high: f.high,
            })
            .collect(),
    };

    let energy_curve_output = EnergyCurveOutput {
        frames:
            (0..energy_curve.overall.len())
            .map(|i| EnergyCurveFrameOutput {
                frame: i,
                time_seconds: frame_time(i, sample_rate),
                overall: energy_curve.overall[i],
                low: energy_curve.low[i],
                mid: energy_curve.mid[i],
                high: energy_curve.high[i],
                delta_overall: energy_curve.delta_overall[i],
                delta_low: energy_curve.delta_low[i],
            })
            .collect(),
        peaks:
            energy_curve.peaks
            .iter()
            .map(|&frame| FramePositionOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
            })
            .collect(),
        valleys:
            energy_curve.valleys
            .iter()
            .map(|&frame| FramePositionOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
            })
            .collect(),
    };

    let onset_output = OnsetOutput {
        frames:
            (0..onset.strength.len())
            .map(|i| OnsetFrameOutput {
                frame: i,
                time_seconds: frame_time(i, sample_rate),
                strength: onset.strength[i],
                smoothed: onset.smoothed[i],
            })
            .collect(),
    };

    let kick_output = KickOutput {
        strength_frames:
            kick.strength
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let frame = index + 1;
                FrameValueOutput {
                    frame,
                    time_seconds: frame_time(frame, sample_rate),
                    value,
                }
            })
            .collect(),
        envelope_frames:
            kick.bandpass_envelope
            .iter()
            .enumerate()
            .map(|(frame, &value)| FrameValueOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
                value,
            })
            .collect(),
    };

    let structure_output = StructureOutput {
        sections:
            energy_curve.sections
            .iter()
            .map(|s| SectionOutput {
                start_frame: s.start,
                start_time_seconds: frame_time(s.start, sample_rate),
                end_frame: s.end,
                end_time_seconds: frame_time(s.end, sample_rate),
                section_type: section_type_label(&s.label).to_string(),
            })
            .collect(),
    };

    let chroma_output = ChromaOutput {
        frames:
            chroma.chroma
            .iter()
            .enumerate()
            .map(|(frame, values)| ChromaFrameOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
                values: *values,
            })
            .collect(),
        overall_key: key_estimate_output(&key.overall.candidates),
        section_keys:
            key.sections
            .iter()
            .map(|sk| SectionKeyOutput {
                start_frame: sk.start,
                end_frame: sk.end,
                key: key_estimate_output(&sk.estimate.candidates),
            })
            .collect(),
    };

    let brightness_output = BrightnessOutput {
        frames:
            brightness.values
            .iter()
            .enumerate()
            .map(|(frame, &value)| FrameValueOutput {
                frame,
                time_seconds: frame_time(frame, sample_rate),
                value,
            })
            .collect(),
    };

    AnalysisOutput {
        version: FORMAT_VERSION,
        audio: audio_output,
        beat: beat_output,
        energy: energy_output,
        energy_curve: energy_curve_output,
        onset: onset_output,
        kick: kick_output,
        structure: structure_output,
        chroma: chroma_output,
        brightness: brightness_output,
    }
}

/// AnalysisOutput の内容を検証し、問題点のリストを返す(空なら問題なし)。
pub fn validate_output(output: &AnalysisOutput) -> Vec<String> {

    let mut issues = Vec::new();

    let expected_frames = output.energy.frames.len();

    // 1. 各frames配列の長さ一致確認
    if output.chroma.frames.len() != expected_frames {
        issues.push(format!(
            "chroma.frames length mismatch: expected {}, got {}",
            expected_frames, output.chroma.frames.len()
        ));
    }

    if output.onset.frames.len() != expected_frames {
        issues.push(format!(
            "onset.frames length mismatch: expected {}, got {}",
            expected_frames, output.onset.frames.len()
        ));
    }

    if output.energy_curve.frames.len() != expected_frames {
        issues.push(format!(
            "energy_curve.frames length mismatch: expected {}, got {}",
            expected_frames, output.energy_curve.frames.len()
        ));
    }

    if output.brightness.frames.len() != expected_frames {
        issues.push(format!(
            "brightness.frames length mismatch: expected {}, got {}",
            expected_frames, output.brightness.frames.len()
        ));
    }

    if output.kick.envelope_frames.len() != expected_frames {
        issues.push(format!(
            "kick.envelope_frames length mismatch: expected {}, got {}",
            expected_frames, output.kick.envelope_frames.len()
        ));
    }

    // 2. NaN / 無限大の検出
    let mut has_non_finite = false;

    for frame in &output.energy.frames {
        if !frame.rms.is_finite() || !frame.low.is_finite()
            || !frame.mid.is_finite() || !frame.high.is_finite() {
            has_non_finite = true;
        }
    }

    for frame in &output.chroma.frames {
        if frame.values.iter().any(|v| !v.is_finite()) {
            has_non_finite = true;
        }
    }

    if !output.beat.bpm.is_finite() {
        has_non_finite = true;
    }

    if has_non_finite {
        issues.push("Non-finite (NaN/Inf) values detected".to_string());
    }

    // 3. 重要な配列が空でないか
    if output.energy.frames.is_empty() {
        issues.push("energy.frames is empty".to_string());
    }

    if output.chroma.frames.is_empty() {
        issues.push("chroma.frames is empty".to_string());
    }

    if output.structure.sections.is_empty() {
        issues.push("structure.sections is empty".to_string());
    }

    // 4. 先頭・末尾のタイムスタンプの妥当性
    if let Some(first) = output.energy.frames.first() {
        if first.time_seconds.abs() > 0.1 {
            issues.push(format!(
                "First frame time_seconds is not near 0.0: {}",
                first.time_seconds
            ));
        }
    }

    if let Some(last) = output.energy.frames.last() {
        let diff = (output.audio.duration_seconds - last.time_seconds).abs();

        if diff > 5.0 {
            issues.push(format!(
                "Last frame time_seconds ({}) is far from audio duration ({})",
                last.time_seconds, output.audio.duration_seconds
            ));
        }
    }

    // 5. 値の範囲チェック
    if output.beat.bpm <= 0.0 || output.beat.bpm > 300.0 {
        issues.push(format!("BPM out of expected range: {}", output.beat.bpm));
    }

    for frame in &output.chroma.frames {
        if frame.values.iter().any(|&v| v < 0.0 || v > 1.0 + 1e-3) {
            issues.push(format!(
                "Chroma value out of [0,1] range at frame {}",
                frame.frame
            ));
            break;
        }
    }

    for frame in &output.brightness.frames {
        if frame.value < 0.0 || frame.value > 1.0 + 1e-3 {
            issues.push(format!(
                "Brightness value out of [0,1] range at frame {}",
                frame.frame
            ));
            break;
        }
    }

    if output.chroma.overall_key.correlation < -1.0
        || output.chroma.overall_key.correlation > 1.0 {
        issues.push(format!(
            "Overall key correlation out of [-1,1] range: {}",
            output.chroma.overall_key.correlation
        ));
    }

    issues
}