use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::audio::raw_audio::RawAudio;
use crate::error::AnalyzerError;

pub fn open_file(path: &Path) -> Result<File, AnalyzerError> {
    let file = File::open(path)?;

    Ok(file)
}


/// WAVヘッダを確認する。
pub fn validate_wav(file: &mut File) -> Result<(), AnalyzerError> {
    let mut header = [0u8; 12];

    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut header)?;

    if &header[0..4] != b"RIFF" {
        return Err(AnalyzerError::InvalidWav);
    }

    if &header[8..12] != b"WAVE" {
        return Err(AnalyzerError::InvalidWav);
    }

    Ok(())
}

#[derive(Debug)]
pub struct WavInfo {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
}

pub fn read_wav_info(file: &mut File) -> Result<WavInfo, AnalyzerError> {
    file.seek(SeekFrom::Start(12))?;

    loop {
        let mut chunk_id = [0u8; 4];
        let mut chunk_size = [0u8; 4];

        file.read_exact(&mut chunk_id)?;
        file.read_exact(&mut chunk_size)?;

        let size = u32::from_le_bytes(chunk_size);

        if &chunk_id == b"fmt " {
            let mut fmt = vec![0u8; size as usize];
            file.read_exact(&mut fmt)?;

            let channels = u16::from_le_bytes([fmt[2], fmt[3]]);
            let sample_rate =
                u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
            let bits_per_sample =
                u16::from_le_bytes([fmt[14], fmt[15]]);

            return Ok(WavInfo {
                channels,
                sample_rate,
                bits_per_sample,
            });
        } else {
            skip_chunk(file, size)?;
        }
    }
}

pub fn read_pcm_data(
    file: &mut File,
    info: &WavInfo,
) -> Result<RawAudio, AnalyzerError> {

    if info.bits_per_sample != 16 {
        return Err(AnalyzerError::UnsupportedBitDepth);
    }

    file.seek(SeekFrom::Start(12))?;

    loop {
        let mut chunk_id = [0u8; 4];
        let mut chunk_size = [0u8; 4];

        file.read_exact(&mut chunk_id)?;
        file.read_exact(&mut chunk_size)?;

        let size = u32::from_le_bytes(chunk_size);

        if &chunk_id == b"data" {

            let mut raw_data = vec![0u8; size as usize];

            file.read_exact(&mut raw_data)?;

            let samples: Vec<f32> = raw_data
                .chunks_exact(2)
                .map(|bytes| {
                    let value =
                        i16::from_le_bytes([bytes[0], bytes[1]]);

                    value as f32 / i16::MAX as f32
                })
                .collect();

            return Ok(RawAudio {
                samples,
                sample_rate: info.sample_rate,
                channels: info.channels,
            });
        }

        file.seek(SeekFrom::Current(size as i64))?;
    }
}

/// チャンクをスキップする。奇数サイズの場合は仕様上のパディング1バイトも読み飛ばす。
fn skip_chunk(file: &mut File, size: u32) -> Result<(), AnalyzerError> {

    let padded_size =
        if size % 2 == 1 {
            size + 1
        } else {
            size
        };

    file.seek(SeekFrom::Current(padded_size as i64))?;

    Ok(())
}