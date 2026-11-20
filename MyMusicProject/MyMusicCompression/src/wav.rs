use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub struct WavReader {
    file: File,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    data_remaining: u64,
    pub data_size: u64,
}

impl WavReader {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open WAV: {e}"))?;

        let mut header = [0u8; 12];
        file.read_exact(&mut header)
            .map_err(|e| format!("Failed to read WAV header: {e}"))?;

        if &header[0..4] != b"RIFF" {
            return Err("Not a RIFF file".to_string());
        }

        if &header[8..12] != b"WAVE" {
            return Err("Not a WAVE file".to_string());
        }

        let mut channels = None;
        let mut sample_rate = None;
        let mut bits_per_sample = None;
        let mut data_size = None;
        let mut data_start = None;

        loop {
            let mut chunk_header = [0u8; 8];

            match file.read_exact(&mut chunk_header) {
                Ok(()) => {}

                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }

                Err(e) => {
                    return Err(format!("Failed to read WAV chunk: {e}"));
                }
            }

            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]) as u64;

            let chunk_data_start = file
                .stream_position()
                .map_err(|e| format!("Failed to get WAV position: {e}"))?;

            match &chunk_header[0..4] {
                b"fmt " => {
                    if chunk_size < 16 {
                        return Err("Invalid fmt chunk".to_string());
                    }

                    let mut fmt = [0u8; 16];

                    file.read_exact(&mut fmt)
                        .map_err(|e| format!("Failed to read fmt chunk: {e}"))?;

                    let audio_format = u16::from_le_bytes([fmt[0], fmt[1]]);

                    if audio_format != 1 {
                        return Err("Only PCM WAV is supported".to_string());
                    }

                    channels = Some(u16::from_le_bytes([fmt[2], fmt[3]]));

                    sample_rate = Some(u32::from_le_bytes([
                        fmt[4], fmt[5], fmt[6], fmt[7],
                    ]));

                    bits_per_sample =
                        Some(u16::from_le_bytes([fmt[14], fmt[15]]));

                    let remaining = chunk_size - 16;

                    if remaining > 0 {
                        file.seek(SeekFrom::Current(remaining as i64))
                            .map_err(|e| {
                                format!("Failed to skip fmt chunk: {e}")
                            })?;
                    }
                }

                b"data" => {
                    data_size = Some(chunk_size);
                    data_start = Some(chunk_data_start);

                    file.seek(SeekFrom::Current(chunk_size as i64))
                        .map_err(|e| {
                            format!("Failed to skip data chunk: {e}")
                        })?;
                }

                _ => {
                    file.seek(SeekFrom::Current(chunk_size as i64))
                        .map_err(|e| {
                            format!("Failed to skip WAV chunk: {e}")
                        })?;
                }
            }

            if chunk_size & 1 != 0 {
                file.seek(SeekFrom::Current(1))
                    .map_err(|e| format!("Failed to skip WAV padding: {e}"))?;
            }

            if channels.is_some()
                && sample_rate.is_some()
                && bits_per_sample.is_some()
                && data_size.is_some()
            {
                break;
            }
        }

        let channels = channels.ok_or("fmt chunk not found")?;
        let sample_rate = sample_rate.ok_or("Sample rate not found")?;
        let bits_per_sample =
            bits_per_sample.ok_or("Bit depth not found")?;
        let data_size = data_size.ok_or("data chunk not found")?;
        let data_start = data_start.ok_or("data chunk position not found")?;

        if bits_per_sample != 16 {
            return Err(format!(
                "Only 16-bit PCM WAV is supported, found {}-bit",
                bits_per_sample
            ));
        }

        if channels == 0 {
            return Err("Invalid channel count".to_string());
        }

        file.seek(SeekFrom::Start(data_start))
            .map_err(|e| format!("Failed to seek to PCM data: {e}"))?;

        Ok(Self {
            file,
            sample_rate,
            channels,
            bits_per_sample,
            data_remaining: data_size,
            data_size,
        })
    }

    pub fn read_samples(
        &mut self,
        samples: &mut [i16],
    ) -> Result<usize, String> {
        if self.data_remaining == 0 {
            return Ok(0);
        }

        let requested_bytes = samples.len() * 2;
        let bytes_to_read =
            requested_bytes.min(self.data_remaining as usize);

        let mut buffer = vec![0u8; bytes_to_read];

        self.file
            .read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read PCM data: {e}"))?;

        let sample_count = bytes_to_read / 2;

        for i in 0..sample_count {
            samples[i] =
                i16::from_le_bytes([buffer[i * 2], buffer[i * 2 + 1]]);
        }

        self.data_remaining -= (sample_count * 2) as u64;

        Ok(sample_count)
    }

    pub fn progress(&self) -> f64 {
        if self.data_size == 0 {
            return 1.0;
        }

        1.0 - (self.data_remaining as f64 / self.data_size as f64)
    }
}