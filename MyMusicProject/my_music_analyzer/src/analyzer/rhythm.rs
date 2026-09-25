use crate::analyzer::energy::EnergyFrame;
use crate::analyzer::features::{
    BeatFeature,
    OnsetFeature,
};

fn normalize(
    value: f32,
    max: f32
) -> f32 {

    if max == 0.0 {
        return 0.0;
    }

    value / max
}

pub fn estimate_period(
    strength: &Vec<f32>,
    fps: f32,
) -> Vec<usize> {

    let min_bpm = 60.0;
    let max_bpm = 200.0;

    let min_lag =
        (fps * 60.0 / max_bpm) as usize;

    let max_lag =
        (fps * 60.0 / min_bpm) as usize;


    let mut correlation =
        vec![0.0; max_lag + 1];


    for lag in min_lag..=max_lag {

        let mut score = 0.0;


        for i in lag..strength.len() {

            score +=
                strength[i]
                *
                strength[i - lag];
        }


        correlation[lag] = score;
    }


    let threshold =
        correlation
            .iter()
            .copied()
            .fold(0.0, f32::max)
            *
            0.5;


    let mut peaks = Vec::new();


    for lag in min_lag + 1..max_lag {

        if correlation[lag] > threshold
            &&
           correlation[lag] > correlation[lag - 1]
            &&
           correlation[lag] > correlation[lag + 1]
        {

            peaks.push(lag);

        }
    }


    for lag in min_lag..=max_lag {

        println!(
            "{:2}: {:.2}",
            lag,
            correlation[lag]
        );

    }


    peaks
}

fn lag_to_bpm(
    lag: usize,
    fps: f32,
) -> f32 {

    let seconds =
        lag as f32 / fps;

    60.0 / seconds
}

fn generate_bpm_candidates(
    periods: &[usize],
    fps: f32,
) -> Vec<f32> {

    let mut candidates = Vec::new();

    for &period in periods {

        let bpm =
            lag_to_bpm(
                period,
                fps
            );

        candidates.push(bpm);
        candidates.push(bpm * 2.0);
        candidates.push(bpm / 2.0);
    }

    let mut candidates =
        candidates
        
            .into_iter()
            .filter(|bpm|
                *bpm >= 60.0 &&
                *bpm <= 180.0
            )
            .collect::<Vec<_>>();


    candidates.sort_by(|a,b|
        a.partial_cmp(b).unwrap()
    );


    candidates.dedup_by(|a,b|
        (*a - *b).abs() < 1.0
    );


    candidates
}

fn interpolate(
    signal: &[f32],
    position: f32,
) -> f32 {

    if position < 0.0
        || position >= signal.len() as f32
    {
        return 0.0;
    }

    let index =
        position.floor() as usize;

    let fraction =
        position - index as f32;

    if index + 1 >= signal.len() {
        return signal[index];
    }

    signal[index] * (1.0 - fraction)
        +
    signal[index + 1] * fraction
}

fn score_bpm_candidate(
    bpm: f32
) -> f32 {

    let mut score = 0.0;


    // EDM中心帯
    if bpm >= 90.0 && bpm <= 150.0 {
        score += 10.0;
    }


    // 特に多いEDM帯
    if bpm >= 120.0 && bpm <= 130.0 {
        score += 5.0;
    }


    // 128 BPM付近
    if (bpm - 128.0).abs() < 5.0 {
        score += 5.0;
    }


    score
}

fn select_best_bpm(
    candidates:&Vec<f32>
)->f32 {

    let mut best_bpm = 0.0;
    let mut best_score = -1.0;


    for &bpm in candidates {

        let score =
            score_bpm_candidate(bpm);


        println!(
            "BPM {:.2} Score {:.2}",
            bpm,
            score
        );


        if score > best_score {

            best_score =
                score;

            best_bpm =
                bpm;
        }
    }


    best_bpm
}

fn correct_bpm(
    estimated_bpm: f32,
    strength: &[f32],
    fps: f32,
) -> f32 {

    let center =
        estimated_bpm.round() as i32;

    let mut best_bpm =
        center;

    let mut best_score =
        -1.0f32;


    // 推定BPMの±2 BPMを評価
    for bpm in (center - 2)..=(center + 2) {

        if bpm <= 0 {
            continue;
        }


        // BPM → 小数フレーム周期
        let period =
            fps * 60.0 / bpm as f32;


        if period <= 0.0
            || period >= strength.len() as f32
        {
            continue;
        }


        let mut score =
            0.0f32;

        let mut count =
            0usize;


        // 小数フレーム周期のまま相関を計算
        let mut position =
            period;

        while position < strength.len() as f32 {

            let current =
                interpolate(
                    strength,
                    position,
                );

            let previous =
                interpolate(
                    strength,
                    position - period,
                );

            score +=
                current * previous;

            count += 1;

            position += 1.0;
        }


        // 比較するため平均化
        if count > 0 {

            score /=
                count as f32;
        }


        println!(
            "BPM {} -> period {:.4} -> score {:.6}",
            bpm,
            period,
            score
        );


        if score > best_score {

            best_score =
                score;

            best_bpm =
                bpm;
        }
    }


    let corrected_bpm =
        best_bpm as f32;


    // 0.5 BPM以上ズレた場合だけ補正
    if (estimated_bpm - corrected_bpm).abs() >= 0.5 {

        println!(
            "BPM correction: {:.2} -> {:.2}",
            estimated_bpm,
            corrected_bpm
        );

        corrected_bpm

    } else {

        corrected_bpm
    }
}

pub fn analyze_rhythm(
    frames: &Vec<EnergyFrame>,
    onset: &OnsetFeature,
    fps: f32,
) -> BeatFeature {


    let periods =
        estimate_period(
            &onset.smoothed,
            fps,
        );

    println!("Detected periods: {:?}", periods);

    let bpm_candidates =
        generate_bpm_candidates(
            &periods,
            fps,
        );

    println!("BPM candidates:");

    for bpm in &bpm_candidates {

        println!("{:.2}", bpm);

    }


    let mut beat_strength = Vec::new();

    let max_low = frames
        .iter()
        .map(|f| f.low)
        .fold(0.0, f32::max);

    let max_rms = frames
        .iter()
        .map(|f| f.rms)
        .fold(0.0, f32::max);

    for i in 1..frames.len() {

        let previous =
            normalize(frames[i-1].low, max_low) * 0.7
            +
            normalize(frames[i-1].rms, max_rms) * 0.3;


        let current =
            normalize(frames[i].low, max_low) * 0.7
            +
            normalize(frames[i].rms, max_rms) * 0.3;


        let difference =
            current - previous;


        if difference > 0.0 {
            beat_strength.push(difference);
        }
        else {
            beat_strength.push(0.0);
        }
    }

    let estimated_bpm =
        select_best_bpm(&bpm_candidates);

    let bpm =
        correct_bpm(
            estimated_bpm,
            &onset.smoothed,
            fps,
        );

    let beats =
        refine_beats(
            &onset.smoothed,
            bpm,
            fps,
        );

    println!("Beats: {:?}", &beats[..beats.len().min(20)]);

    BeatFeature {
        bpm,
        bpm_candidates,
        beats,
        beat_strength,
    }
}

fn refine_beats(
    onset: &[f32],
    bpm: f32,
    fps: f32,
) -> Vec<usize> {

    if onset.is_empty() || bpm <= 0.0 || fps <= 0.0 {
        return Vec::new();
    }


    // BPM → ビート間のフレーム数
    let period =
        fps * 60.0 / bpm;


    // 理想位置の周辺を探索する幅
    // ±20% の範囲
    let search_range =
        (period * 0.2).round() as usize;


    let mut beats =
        Vec::new();


    // 最初のビートを探す
    let first_end =
        search_range.min(onset.len());


    let mut first =
        0usize;

    for i in 0..first_end {

        if onset[i] > onset[first] {
            first = i;
        }
    }


    beats.push(first);


    let mut expected =
        first as f32 + period;


    while expected < onset.len() as f32 {

        let center =
            expected.round() as isize;


        let start =
            (center - search_range as isize)
            .max(0) as usize;

        let end =
            (center + search_range as isize)
            .min(onset.len() as isize - 1)
            as usize;


        let mut best =
            start;


        for i in start..=end {

            if onset[i] > onset[best] {
                best = i;
            }
        }


        beats.push(best);


        expected += period;
    }


    beats
}