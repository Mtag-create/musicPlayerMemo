use crate::analyzer::energy::EnergyFrame;
use crate::analyzer::features::EnergyCurveFeature;
use crate::analyzer::features::SectionFeature;
use crate::analyzer::framing;

const SMOOTH_WINDOW: usize = 20;

fn normalize(value: f32, max: f32) -> f32 {
    if max == 0.0 {
        return 0.0;
    }

    value / max
}

/// EnergyFrame列から、正規化・加重和・平滑化を行った
/// 「エネルギー曲線」(帯域ごと4本 + 総合1本)を生成する。
pub fn analyze_energy_curve(frames: &Vec<EnergyFrame>) -> EnergyCurveFeature {

    let max_rms = frames.iter().map(|f| f.rms).fold(0.0, f32::max);
    let max_low = frames.iter().map(|f| f.low).fold(0.0, f32::max);
    let max_mid = frames.iter().map(|f| f.mid).fold(0.0, f32::max);
    let max_high = frames.iter().map(|f| f.high).fold(0.0, f32::max);

    let mut norm_rms = Vec::new();
    let mut norm_low = Vec::new();
    let mut norm_mid = Vec::new();
    let mut norm_high = Vec::new();
    let mut overall = Vec::new();

    for frame in frames {

        let rms = normalize(frame.rms, max_rms);
        let low = normalize(frame.low, max_low);
        let mid = normalize(frame.mid, max_mid);
        let high = normalize(frame.high, max_high);

        let combined =
            rms * 0.35
            + low * 0.35
            + mid * 0.20
            + high * 0.10;

        norm_rms.push(rms);
        norm_low.push(low);
        norm_mid.push(mid);
        norm_high.push(high);
        overall.push(combined);
    }
let rms = framing::moving_average(&norm_rms, SMOOTH_WINDOW);
    let low = framing::moving_average(&norm_low, SMOOTH_WINDOW);
    let mid = framing::moving_average(&norm_mid, SMOOTH_WINDOW);
    let high = framing::moving_average(&norm_high, SMOOTH_WINDOW);
    let overall = framing::moving_average(&overall, SMOOTH_WINDOW);

    let (peaks, valleys) = detect_peaks_and_valleys(&overall);

    let sections = build_sections(&overall, &peaks, &valleys);
    let sections = merge_short_sections(sections, MIN_SECTION_LENGTH);

    let delta_overall = calculate_delta(&overall, LOOKBACK_FRAMES);
    let delta_low = calculate_delta(&low, LOOKBACK_FRAMES);

    EnergyCurveFeature {
        rms,
        low,
        mid,
        high,
        overall,
        peaks,
        valleys,
        sections,
        delta_overall,
        delta_low,
    }
}

const PEAK_WINDOW: usize = 40;
const PEAK_THRESHOLD: f32 = 0.08;
const MIN_SECTION_LENGTH: usize = 172;

/// 平滑化済みの overall 曲線から山(極大)・谷(極小)を検出する。
///
/// 前後 PEAK_WINDOW フレームの範囲内で最大/最小であり、
/// かつ隣接点との差が PEAK_THRESHOLD 以上ある点のみを採用する。
fn detect_peaks_and_valleys(overall: &Vec<f32>) -> (Vec<usize>, Vec<usize>) {

    let mut peaks = Vec::new();
    let mut valleys = Vec::new();

    if overall.len() <= PEAK_WINDOW * 2 {
        return (peaks, valleys);
    }

    for i in PEAK_WINDOW..overall.len() - PEAK_WINDOW {

        let start = i - PEAK_WINDOW;
        let end = i + PEAK_WINDOW;

        let window = &overall[start..=end];

        let max = window.iter().cloned().fold(f32::MIN, f32::max);
        let min = window.iter().cloned().fold(f32::MAX, f32::min);

        let value = overall[i];

        let range = max - min;

        if range < PEAK_THRESHOLD {
            continue;
        }

        if value == max {
            peaks.push(i);
        } else if value == min {
            valleys.push(i);
        }
    }

    (peaks, valleys)
}

const DROP_PERCENTILE: f32 = 0.30;

/// 検出済みの山(peaks)の overall 値から、
/// 上位 DROP_PERCENTILE に相当する値の閾値を求める。
fn calculate_drop_threshold(overall: &Vec<f32>, peaks: &Vec<usize>) -> f32 {

    if peaks.is_empty() {
        return f32::MAX;
    }

    let mut values: Vec<f32> =
        peaks.iter().map(|&i| overall[i]).collect();

    values.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let count =
        ((values.len() as f32) * DROP_PERCENTILE).ceil() as usize;

    let count = count.max(1).min(values.len());

    values[count - 1]
}

/// 指定範囲 (exclusive な start_excl, end_excl) 内で、
/// overall値が最大となる peak のインデックスを返す。
fn find_representative_peak(
    peaks: &Vec<usize>,
    overall: &Vec<f32>,
    start_excl: usize,
    end_excl: usize,
) -> Option<usize> {

    peaks
        .iter()
        .filter(|&&p| p > start_excl && p < end_excl)
        .cloned()
        .max_by(|&a, &b| overall[a].partial_cmp(&overall[b]).unwrap())
}

/// 山谷の位置関係から、曲を意味のあるセクションに分割する。
fn build_sections(
    overall: &Vec<f32>,
    peaks: &Vec<usize>,
    valleys: &Vec<usize>,
) -> Vec<SectionFeature> {

    let mut sections = Vec::new();

    if overall.is_empty() {
        return sections;
    }

    if valleys.is_empty() {
        // 谷が1つも検出されなかった場合は全体を1セクション扱いにする
        sections.push(SectionFeature {
            label: "維持".to_string(),
            start: 0,
            end: overall.len() - 1,
        });
        return sections;
    }

    let drop_threshold = calculate_drop_threshold(overall, peaks);

    // イントロ: 曲頭 〜 最初の谷
    sections.push(SectionFeature {
        label: "イントロ".to_string(),
        start: 0,
        end: valleys[0],
    });

    // 谷ペアごとの区間
    for window in valleys.windows(2) {

        let start = window[0];
        let end = window[1];

        match find_representative_peak(peaks, overall, start, end) {

            Some(peak) => {

                sections.push(SectionFeature {
                    label: "ビルドアップ".to_string(),
                    start,
                    end: peak,
                });

                let second_label =
                    if overall[peak] >= drop_threshold {
                        "ドロップ"
                    } else {
                        "ブレイクダウン"
                    };

                sections.push(SectionFeature {
                    label: second_label.to_string(),
                    start: peak,
                    end,
                });
            }

            None => {

                sections.push(SectionFeature {
                    label: "維持".to_string(),
                    start,
                    end,
                });
            }
        }
    }

    // アウトロ: 最後の谷 〜 曲末尾
    sections.push(SectionFeature {
        label: "アウトロ".to_string(),
        start: *valleys.last().unwrap(),
        end: overall.len() - 1,
    });

    sections
}

/// 一定の長さに満たない短いセクションを、
/// 直前・直後のうちより長い方に統合する。
fn merge_short_sections(
    mut sections: Vec<SectionFeature>,
    min_length: usize,
) -> Vec<SectionFeature> {

    loop {

        let short_index = sections
            .iter()
            .position(|s| s.end - s.start < min_length);

        let index = match short_index {
            Some(i) => i,
            None => break,
        };

        // セクションが1つしかない場合はマージ不要
        if sections.len() <= 1 {
            break;
        }

        let has_prev = index > 0;
        let has_next = index + 1 < sections.len();

        let merge_into_prev =
            if has_prev && has_next {

                let prev_len =
                    sections[index - 1].end - sections[index - 1].start;

                let next_len =
                    sections[index + 1].end - sections[index + 1].start;

                prev_len >= next_len

            } else {
                has_prev
            };

        if merge_into_prev {

            sections[index - 1].end = sections[index].end;
            sections.remove(index);

        } else {

            sections[index + 1].start = sections[index].start;
            sections.remove(index);
        }
    }

    sections
}

const LOOKBACK_FRAMES: usize = 10;

/// lookback フレーム前との差分(符号付き)を計算する。
/// 正: 上昇中、負: 下降中。比較対象がない先頭部分は 0.0。
fn calculate_delta(curve: &Vec<f32>, lookback: usize) -> Vec<f32> {

    curve
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            if i >= lookback {
                value - curve[i - lookback]
            } else {
                0.0
            }
        })
        .collect()
}