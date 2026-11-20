mod wav;
mod encoder;

use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let input_dir = Path::new(r"E:\Trimmed");
    let output_dir = Path::new(r"E:\DeviceMusic");

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {e}");
        return;
    }

    let mut wav_files: Vec<PathBuf> = match fs::read_dir(input_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
            })
            .collect(),

        Err(e) => {
            eprintln!("Failed to read input directory: {e}");
            return;
        }
    };

    wav_files.sort();

    println!("Found {} WAV files", wav_files.len());

    wav_files.par_iter().for_each(|input| {
        let file_name = match input.file_stem() {
            Some(name) => name,
            None => {
                eprintln!("Skipping invalid filename: {}", input.display());
                return;
            }
        };

        let output = output_dir.join(format!("{}.mp3", file_name.to_string_lossy()));

        println!("[START] {}", input.display());

        let mut reader = match wav::WavReader::open(input) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("[ERROR] {}: {e}", input.display());
                return;
            }
        };

        match encoder::encode_mp3(&mut reader, &output) {
            Ok(()) => {
                println!("[DONE]  {}", output.display());
            }

            Err(e) => {
                eprintln!("[ERROR] {}: {e}", input.display());
            }
        }
    });

    println!();
    println!("All files processed.");
}