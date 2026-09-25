use std::env;
use std::path::PathBuf;

use my_music_analyzer::database::writer;

fn main() {

    let mut args = env::args();
    args.next(); // 実行ファイル名を読み飛ばす

    let input_path = match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: decode <path-to-mmz-file>");
            return;
        }
    };

    let output = match writer::load_binary(&input_path) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Error loading {}: {}", input_path.display(), error);
            return;
        }
    };

    let issues = writer::validate_output(&output);

    if issues.is_empty() {
        println!("Validation: OK");
    } else {
        println!("Validation issues:");
        for issue in &issues {
            println!("  - {}", issue);
        }
    }

    let json_string = match serde_json::to_string_pretty(&output) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Error serializing to JSON: {}", error);
            return;
        }
    };

    let output_path = input_path.with_extension("json");

    match std::fs::write(&output_path, json_string) {
        Ok(()) => println!("Decoded JSON saved: {}", output_path.display()),
        Err(error) => eprintln!("Error saving JSON: {}", error),
    }
}