use hound::{WavReader, WavSpec, WavWriter};
use std::fs;
use std::path::{Path, PathBuf};

const INPUT_DIR: &str = r"E:\Input";
const OUTPUT_DIR: &str = r"E:\Trimmed";

// 無音判定の閾値
// 0.001 ≒ -60 dBFS
const SILENCE_THRESHOLD: f32 = 0.0055;

// この時間以上続く低音量区間だけトリミング対象にする
const MIN_SILENCE_SECONDS: f32 = 0.05;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)?;

    for entry in fs::read_dir(INPUT_DIR)? {
        let path = entry?.path();

        if path.extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("wav"))
            != Some(true)
        {
            continue;
        }

        trim_wav(&path)?;
    }

    println!("完了！");
    Ok(())
}

fn trim_wav(input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(input_path)?;
    let spec = reader.spec();

    let samples: Vec<i32> = reader.samples::<i32>()
        .collect::<Result<Vec<_>, _>>()?;

    let channels = spec.channels as usize;

    if channels == 0 {
        return Ok(());
    }

    let total_frames = samples.len() / channels;

    if total_frames == 0 {
        return Ok(());
    }

    let max_amplitude = match spec.bits_per_sample {
        8 => 128.0,
        16 => 32768.0,
        24 => 8388608.0,
        32 => 2147483648.0,
        bits => return Err(format!("未対応のビット深度: {}", bits).into()),
    };

    let threshold = SILENCE_THRESHOLD * max_amplitude;

    // フレーム単位で音量判定
    let is_silent = |frame: usize| -> bool {
        let start = frame * channels;
        let end = start + channels;

        samples[start..end]
            .iter()
            .all(|&sample| (sample as f32).abs() < threshold)
    };

    let min_silent_frames =
        (MIN_SILENCE_SECONDS * spec.sample_rate as f32) as usize;

    // 先頭の無音
    let mut start_frame = 0;

    while start_frame < total_frames && is_silent(start_frame) {
        start_frame += 1;
    }

    // 末尾の無音
    let mut end_frame = total_frames;

    while end_frame > start_frame && is_silent(end_frame - 1) {
        end_frame -= 1;
    }

    // 短すぎる無音は削らない
    if start_frame < min_silent_frames {
        start_frame = 0;
    }

    if total_frames - end_frame < min_silent_frames {
        end_frame = total_frames;
    }

    let start_sample = start_frame * channels;
    let end_sample = end_frame * channels;

    let output_name = input_path
        .file_name()
        .ok_or("ファイル名取得失敗")?;

    let output_path = PathBuf::from(OUTPUT_DIR).join(output_name);

    let mut writer = WavWriter::create(&output_path, spec)?;

    for &sample in &samples[start_sample..end_sample] {
        writer.write_sample(sample)?;
    }

    writer.finalize()?;

    let start_seconds = start_frame as f64 / spec.sample_rate as f64;
    let end_seconds = end_frame as f64 / spec.sample_rate as f64;
    let original_seconds = total_frames as f64 / spec.sample_rate as f64;

    println!(
        "{}\n  {:.3}s → {:.3}s\n  {:.3}s → {:.3}s\n",
        output_name.to_string_lossy(),
        start_seconds,
        original_seconds - end_seconds,
        original_seconds,
        end_seconds - start_seconds
    );

    Ok(())
}