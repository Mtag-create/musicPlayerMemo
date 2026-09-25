use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use my_music_analyzer::audio::mono_audio;
use my_music_analyzer::error::AnalyzerError;
use my_music_analyzer::{analyzer, config, database, input};
struct AnalysisSummary {
    bpm: f32,
    section_count: usize,
    output_path: PathBuf,
}

fn output_path_for(input_path: &Path) -> PathBuf {

    let file_name =
        input_path.file_name()
        .expect("input path has no file name");

    let mut path = PathBuf::from(config::OUTPUT_DIR);

    path.push(file_name);
    path.set_extension("mmz");

    path
}

fn summary_path_for(input_path: &Path) -> PathBuf {

    let file_name =
        input_path.file_name()
        .expect("input path has no file name");

    let mut path = PathBuf::from(config::SUMMARY_DIR);

    path.push(file_name);
    path.set_extension("mmz");

    path
}

fn analyze_file(input_path: &Path) -> Result<AnalysisSummary, AnalyzerError> {

    input::file_loader::validate_file(input_path)?;

    let mut file = input::decoder::open_file(input_path)?;

    input::decoder::validate_wav(&mut file)?;

    let wav_info = input::decoder::read_wav_info(&mut file)?;

    let audio_data =
        input::decoder::read_pcm_data(&mut file, &wav_info)?;

    let mono_audio = mono_audio::to_mono(&audio_data);

    let rms =
        analyzer::feature_extractor::calculate_rms(&mono_audio);

    let fft_frames =
        analyzer::fft::calculate_fft_frames(&mono_audio);

    let fps =
        audio_data.sample_rate as f32 / config::HOP_SIZE as f32;

    let band =
        analyzer::band_energy::calculate_band_energy(&fft_frames);

    let energy_frames =
        analyzer::energy::merge_energy(&rms, &band);

    let spectral =
        analyzer::spectral::calculate_spectral_features(&fft_frames);

    let onset =
        analyzer::onset::analyze_onset(&energy_frames, &spectral);

    let rhythm =
        analyzer::rhythm::analyze_rhythm(&energy_frames, &onset, fps);

    let kick =
        analyzer::kick::analyze_kick(
            &energy_frames,
            &fft_frames,
            mono_audio.sample_rate,
        );

    let energy_curve =
        analyzer::energy_curve::analyze_energy_curve(&energy_frames);

    let chroma =
        analyzer::chroma::analyze_chroma(&mono_audio);

    let key =
        analyzer::key::analyze_key(&chroma.chroma, &energy_curve.sections);

    let brightness =
        analyzer::brightness::calculate_brightness(
            &spectral,
            mono_audio.sample_rate,
        );

    let output =
        database::writer::build_output(
            &mono_audio,
            &rms,
            &energy_frames,
            &onset,
            &rhythm,
            &kick,
            &energy_curve,
            &chroma,
            &key,
            &brightness,
        );

    let output_path = output_path_for(input_path);

    database::writer::save_binary(&output, &output_path)?;

    let summary = mmz_format::compute_summary(&output);
    let summary_path = summary_path_for(input_path);
    mmz_format::save_summary(&summary, &summary_path)?;

    let reloaded = database::writer::load_binary(&output_path)?;

    let issues = database::writer::validate_output(&reloaded);

    if !issues.is_empty() {
        eprintln!(
            "[{}] Validation issues: {:?}",
            output_path.display(),
            issues
        );
    }

    Ok(AnalysisSummary {
        bpm: rhythm.bpm,
        section_count: energy_curve.sections.len(),
        output_path,
    })
}

fn find_wav_files(dir: &Path) -> Vec<PathBuf> {

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("Error reading input directory: {}", error);
            return Vec::new();
        }
    };

    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("wav"))
                    .unwrap_or(false)
        })
        .collect()
}

fn main() {

    println!("{} v{}", config::APP_NAME, config::APP_VERSION);

    let input_dir = Path::new(config::INPUT_DIR);

    let wav_files = find_wav_files(input_dir);

    println!(
        "Found {} WAV file(s) in {}",
        wav_files.len(),
        input_dir.display()
    );

    wav_files.par_iter().for_each(|path| {

        let file_name =
            path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        println!("[{}] Analyzing...", file_name);

        match analyze_file(path) {
            Ok(summary) => {
                println!(
                    "[{}] Done. BPM: {:.2}, Sections: {}, Output: {}",
                    file_name,
                    summary.bpm,
                    summary.section_count,
                    summary.output_path.display()
                );
            }
            Err(error) => {
                eprintln!("[{}] Error: {}", file_name, error);
            }
        }
    });

    println!("Batch analysis complete.");
}