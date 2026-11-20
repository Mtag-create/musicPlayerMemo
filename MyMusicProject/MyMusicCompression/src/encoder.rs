use std::fs::File;
use std::io::Write;
use std::path::Path;

use mp3lame_encoder::{
    Bitrate,
    Builder,
    DualPcm,
    FlushNoGap,
    Quality,
};

use crate::wav::WavReader;

pub fn encode_mp3(
    reader: &mut WavReader,
    output_path: &Path,
) -> Result<(), String> {
    if reader.channels != 2 {
        return Err("Only stereo input is supported".to_string());
    }

    let mut builder =
        Builder::new().ok_or("Failed to create MP3 encoder")?;

    builder
        .set_sample_rate(reader.sample_rate)
        .map_err(|e| format!("Failed to set sample rate: {e:?}"))?;

    builder
        .set_num_channels(2)
        .map_err(|e| format!("Failed to set channel count: {e:?}"))?;

    builder
        .set_brate(Bitrate::Kbps192)
        .map_err(|e| format!("Failed to set bitrate: {e:?}"))?;

    builder
        .set_quality(Quality::Best)
        .map_err(|e| format!("Failed to set encoder quality: {e:?}"))?;

    let mut encoder = builder
        .build()
        .map_err(|e| format!("Failed to build MP3 encoder: {e:?}"))?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {e}"))?;

    const FRAME_SAMPLES: usize = 1152;
    const STEREO_SAMPLES: usize = FRAME_SAMPLES * 2;

    let mut samples = vec![0i16; STEREO_SAMPLES];
    let mut left = Vec::with_capacity(FRAME_SAMPLES);
    let mut right = Vec::with_capacity(FRAME_SAMPLES);
    let mut mp3_buffer = Vec::new();

    loop {
        let sample_count = reader.read_samples(&mut samples)?;

        if sample_count == 0 {
            break;
        }

        left.clear();
        right.clear();

        for stereo_sample in samples[..sample_count].chunks_exact(2) {
            left.push(stereo_sample[0]);
            right.push(stereo_sample[1]);
        }

        if left.is_empty() {
            break;
        }

        let pcm = DualPcm {
            left: &left,
            right: &right,
        };

        mp3_buffer.clear();

        mp3_buffer.reserve(
            mp3lame_encoder::max_required_buffer_size(left.len())
        );

        let encoded_size = encoder
            .encode(pcm, mp3_buffer.spare_capacity_mut())
            .map_err(|e| format!("MP3 encoding failed: {e:?}"))?;

        unsafe {
            mp3_buffer.set_len(encoded_size);
        }

        output
            .write_all(&mp3_buffer)
            .map_err(|e| format!("Failed to write MP3: {e}"))?;
    }

    let mut flush_buffer = vec![0u8; 7200];

    let flush_size = encoder
        .flush::<FlushNoGap>(
            unsafe {
                std::slice::from_raw_parts_mut(
                    flush_buffer.as_mut_ptr()
                        as *mut std::mem::MaybeUninit<u8>,
                    flush_buffer.len(),
                )
            },
        )
        .map_err(|e| format!("MP3 flush failed: {e:?}"))?;

    output
        .write_all(&flush_buffer[..flush_size])
        .map_err(|e| format!("Failed to write final MP3 data: {e}"))?;

    Ok(())
}