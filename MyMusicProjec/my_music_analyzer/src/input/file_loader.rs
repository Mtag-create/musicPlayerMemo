use std::path::Path;

use crate::error::AnalyzerError;

/// 指定されたファイルを検証する。
///
/// 確認内容:
/// - ファイルが存在するか
/// - 対応形式(WAV)か
pub fn validate_file(path: &Path) -> Result<(), AnalyzerError> {
    if !path.exists() {
        return Err(AnalyzerError::FileNotFound);
    }

    if !is_supported_format(path) {
        return Err(AnalyzerError::UnsupportedFormat);
    }

    Ok(())
}

/// 対応している音楽ファイル形式か判定する。
fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("wav"))
        .unwrap_or(false)
}