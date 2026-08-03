// src/audio/mono_audio.rs

use crate::audio::raw_audio::RawAudio;

/// モノラル化済みPCMデータ。
#[derive(Debug)]
pub struct MonoAudio {
    /// モノラル化済みサンプル(-1.0 ～ 1.0)
    pub samples: Vec<f32>,

    /// サンプルレート
    pub sample_rate: u32,
}

/// RawAudio(インターリーブ済み原音データ)からモノラル音声を生成する。
///
/// ステレオ以上のチャンネルは、フレームごとの単純平均でダウンミックスする。
/// モノラル入力の場合はそのままコピーする。
pub fn to_mono(raw: &RawAudio) -> MonoAudio {
    let channels = raw.channels as usize;

    if channels == 0 {
        return MonoAudio {
            samples: Vec::new(),
            sample_rate: raw.sample_rate,
        };
    }

    if channels == 1 {
        return MonoAudio {
            samples: raw.samples.clone(),
            sample_rate: raw.sample_rate,
        };
    }

    let samples: Vec<f32> = raw
        .samples
        .chunks_exact(channels)
        .map(|frame| {
            let sum: f32 = frame.iter().sum();
            sum / channels as f32
        })
        .collect();

    MonoAudio {
        samples,
        sample_rate: raw.sample_rate,
    }
}