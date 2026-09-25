//! PCでの実行テスト用エントリポイント
//!
//! 本番のAndroid処理(JNI経由の呼び出し)とは別の、
//! Rustアルゴリズム単体を実データで検証するための実行入口。
//!
//! 使い方: algorithm_pc_test <.mmzファイルのパス> [<.mmzファイルのパス> ...]
//! 複数指定すると、それぞれを別トラック(IDにインデックスを付与)として
//! ライブラリに読み込む。テスト用の措置であり、本番のRust側ロジックとは無関係。

use algorithm::data::import::load_track;
use algorithm::data::library::Library;
use algorithm::selection::candidate::generate_candidates;
use algorithm::selection::flow::short_wave::ShortWave;
use algorithm::selection::flow::big_wave::BigWave;
use algorithm::selection::flow::curve::EnergyCurve;
use algorithm::selection::scoring::classify_key_relation;
use algorithm::selection::scoring::evaluate_candidates;
use algorithm::selection::flow::trend::{analyze_energy_trend, energy_axis_importance};
use algorithm::selection::scoring::energy_score;
use algorithm::selection::scoring::total_score;
use algorithm::selection::flow::trend::{analyze_bpm_trend, bpm_axis_importance};
use algorithm::selection::flow::trend::{analyze_brightness_trend, brightness_axis_importance};
use algorithm::selection::flow::trend::{analyze_key_stability, key_axis_importance};
use algorithm::selection::history::{final_score, history_bonus};
use algorithm::selection::shortlist::shortlist;
use algorithm::selection::selector::select_next;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let paths = &args[1..];
    if paths.is_empty() {
        eprintln!("使い方: algorithm_pc_test <.mmzファイルのパス> [<.mmzファイルのパス> ...]");
        std::process::exit(1);
    }

    let mut tracks = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("ファイル読み込みエラー({}): {}", path, e);
                std::process::exit(1);
            }
        };

        // テスト用: 同じファイルを複数渡した場合でもIDが重複しないようインデックスを付与
        let id = format!("track_{}", index);

        match load_track(id, &bytes) {
            Ok(track) => {
                println!(
                    "loaded: id={} bpm={} key={}",
                    track.id, track.bpm, track.key.name
                );
                tracks.push(track);
            }
            Err(e) => {
                eprintln!("解析データの読み込みに失敗しました({}): {}", path, e);
                std::process::exit(1);
            }
        }
    }

    let library = Library::new(tracks);

    // 現在再生中の曲がない状態を想定した候補生成の確認
    let candidates = generate_candidates(&library, None);
    println!("candidates (current_track_id=None): {}", candidates.len());

    // 現在再生中の曲がある想定(track_0を除外)
    let candidates_excluding_first = generate_candidates(&library, Some("track_0"));
    println!(
        "candidates (current_track_id=Some(\"track_0\")): {}",
        candidates_excluding_first.len()
    );

    // ShortWaveの動作確認: 8曲分pushして7曲に切り詰められるか確認
    let mut short_wave = ShortWave::new();
    for i in 0..8 {
        short_wave.push(format!("test_track_{}", i));
    }
    println!("short_wave tracks ({}件): {:?}", short_wave.tracks().len(), short_wave.tracks());

    // BigWaveの動作確認: 50曲分pushして49曲に切り詰められるか確認
    let mut big_wave = BigWave::new();
    for i in 0..50 {
        big_wave.push(format!("test_track_{}", i));
    }
    println!("big_wave tracks件数: {}", big_wave.tracks().len());
    println!("big_wave first: {:?}", big_wave.tracks().front());
    println!("big_wave last: {:?}", big_wave.tracks().back());

    // EnergyCurveの動作確認: flat(0.5)で初期化し、長さと任意位置の値を確認
    let energy_curve = EnergyCurve::new_flat(0.5);
    println!("energy_curve len: {}", energy_curve.len());
    println!("energy_curve target_at(0): {:?}", energy_curve.target_at(0));
    println!("energy_curve target_at(48): {:?}", energy_curve.target_at(48));
    println!("energy_curve target_at(49): {:?}", energy_curve.target_at(49));

    // classify_key_relationの動作確認
    let key_pairs = [
        ("D Major", "D Major"),  // Exact
        ("D Major", "B Minor"),  // Relative
        ("D Major", "D Minor"),  // Parallel
        ("D Major", "A Major"),  // Fifth
        ("D Major", "F Major"),  // Other
    ];
    for (a, b) in key_pairs {
        let relation = classify_key_relation(a, b);
        println!("{} × {} -> {:?}", a, b, relation);
    }

    // evaluate_candidatesの動作確認: track_0を現在曲とし、他の候補とのKey相性を一覧表示
    if let Some(current) = library.tracks().first() {
        let candidates = generate_candidates(&library, Some(&current.id));
        let evaluations = evaluate_candidates(&candidates, current, &energy_curve, big_wave.current_position());

        println!("--- Key relation to current track ({}) ---", current.id);
        for eval in &evaluations {
            println!(
                "{} (key={}, bpm={}) -> key_relation={:?}, bpm_diff={:.2}, energy_transition_diff={:.4}, energy_curve_fit_diff={:?}, brightness_transition_diff={:.4}",
                eval.track.id,
                eval.track.key.name,
                eval.track.bpm,
                eval.key_relation,
                eval.bpm_difference,
                eval.energy_transition_difference,
                eval.energy_curve_fit_difference,
                eval.brightness_transition_difference
            );
        }
        // analyze_energy_trendの動作確認①: 実データ7曲(track_0〜track_6)を積んだShortWaveで分析
        let mut short_wave_real = ShortWave::new();
        for i in 0..7 {
            short_wave_real.push(format!("track_{}", i));
        }

        let trend_real = analyze_energy_trend(&short_wave_real, &library);
        println!("energy trend (real 7 tracks): {:?}", trend_real);
        println!("energy axis importance: {}", energy_axis_importance(trend_real));

        // analyze_energy_trendの動作確認②: 存在しないTrackIdを積んだShortWave(分析不能想定)
        let trend_missing = analyze_energy_trend(&short_wave, &library);
        println!("energy trend (missing tracks): {:?}", trend_missing);

        // energy_scoreの動作確認: トレンド分析結果(重要度)を実際のスコアに反映し、
        // 候補の順位がどう変わるかを確認する
        let importance = energy_axis_importance(trend_real);

        let mut scored: Vec<(String, f32)> = evaluations
            .iter()
            .map(|eval| (eval.track.id.clone(), energy_score(eval, importance)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("--- energy_score (importance={:.2}) ---", importance);
        for (id, score) in &scored {
            println!("{}: {:.4}", id, score);
        }

        // 比較用: 重要度を1.0(基準)に固定した場合の順位
        let scored_baseline: Vec<(String, f32)> = {
            let mut s: Vec<(String, f32)> = evaluations
                .iter()
                .map(|eval| (eval.track.id.clone(), energy_score(eval, 1.0)))
                .collect();
            s.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            s
        };
        println!("--- energy_score (importance=1.0, baseline) ---");
        for (id, score) in &scored_baseline {
            println!("{}: {:.4}", id, score);
        }

        // analyze_bpm_trendの動作確認: track_0〜track_6(実データ)で分析
        let bpm_trend_real = analyze_bpm_trend(&short_wave_real, &library);
        println!("bpm trend (real 7 tracks): {:?}", bpm_trend_real);
        println!("bpm axis importance: {}", bpm_axis_importance(bpm_trend_real));

        // --- 検証: Short Waveの中身を変えると、total_scoreの順位が実際に変わるか ---

        // Energyがほぼ変化しない(Sustaining)履歴を人工的に作る。
        // ソート済みリストの先頭7曲(値が近いもの同士)を選ぶ
        let mut sorted_by_energy: Vec<&algorithm::data::track::Track> = library.tracks().iter().collect();
        sorted_by_energy.sort_by(|a, b| {
            a.energy_summary
                .overall
                .partial_cmp(&b.energy_summary.overall)
                .unwrap()
        });
        let mut short_wave_sustaining = ShortWave::new();
        for track in sorted_by_energy.iter().take(7) {
            short_wave_sustaining.push(track.id.clone());
            println!("sustaining history pick: {} (overall={:.4})", track.id, track.energy_summary.overall);
        }
        let trend_sustaining = analyze_energy_trend(&short_wave_sustaining, &library);
        println!("energy trend (sustaining attempt): {:?}", trend_sustaining);

        let importance_fluctuating = energy_axis_importance(trend_real); // 既存: track_0〜6、Fluctuating
        let importance_sustaining = energy_axis_importance(trend_sustaining);

        let bpm_trend = analyze_bpm_trend(&short_wave_real, &library);
        let bpm_importance = bpm_axis_importance(bpm_trend);
        println!("bpm trend (used for total_score): {:?}, importance={}", bpm_trend, bpm_importance);

        let brightness_trend = analyze_brightness_trend(&short_wave_real, &library);
        let brightness_importance = brightness_axis_importance(brightness_trend);
        println!("brightness trend (used for total_score): {:?}, importance={}", brightness_trend, brightness_importance);

        let key_stability = analyze_key_stability(&short_wave_real, &library);
        let key_importance = key_axis_importance(key_stability);
        println!("key stability (average weight): {:?}, importance={}", key_stability, key_importance);

        let mut total_fluctuating: Vec<(String, f32)> = evaluations
            .iter()
            .map(|eval| (eval.track.id.clone(), total_score(eval, importance_fluctuating, bpm_importance, brightness_importance, key_importance)))
            .collect();
        total_fluctuating.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut total_sustaining: Vec<(String, f32)> = evaluations
            .iter()
            .map(|eval| (eval.track.id.clone(), total_score(eval, importance_sustaining, bpm_importance, brightness_importance, key_importance)))
            .collect();
        total_sustaining.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("--- total_score (energy_importance={:.2}, Fluctuating) ---", importance_fluctuating);
        for (id, score) in &total_fluctuating {
            println!("{}: {:.4}", id, score);
        }
        println!("--- total_score (energy_importance={:.2}, Sustaining) ---", importance_sustaining);
        for (id, score) in &total_sustaining {
            println!("{}: {:.4}", id, score);
        }
        // --- 検証: 履歴補正(history_bonus)を加えると順位がどう変わるか ---

        // track_15を直前(distance=1)、track_9を少し前(distance=3)に見立てたBigWaveを作る
        let mut big_wave_history_test = BigWave::new();
        big_wave_history_test.push("track_9".to_string());  // 3曲前
        big_wave_history_test.push("dummy_a".to_string());
        big_wave_history_test.push("dummy_b".to_string());
        big_wave_history_test.push("track_15".to_string()); // 直前

        let mut final_scores: Vec<(String, f32, f32, f32)> = evaluations
            .iter()
            .map(|eval| {
                let ts = total_score(eval, importance_fluctuating, bpm_importance, brightness_importance, key_importance);
                let hb = history_bonus(&big_wave_history_test, &eval.track.id);
                let fs = final_score(ts, hb);
                (eval.track.id.clone(), ts, hb, fs)
            })
            .collect();
        final_scores.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

        println!("--- final_score (with history_bonus, track_15=直前, track_9=3曲前) ---");
        for (id, ts, hb, fs) in &final_scores {
            println!("{}: total={:.4}, history_bonus={:.4}, final={:.4}", id, ts, hb, fs);
        }
        // --- 検証A: 候補の大部分(track_1〜track_15)を既再生、残り(track_16〜track_21)を未再生とした場合 ---
        let mut big_wave_mostly_played = BigWave::new();
        for i in 1..=15 {
            big_wave_mostly_played.push(format!("track_{}", i));
        }
        // track_15が直前(distance=1)、track_1が最も古い(distance=15)
        // track_16〜track_21は一度もpushしない(未再生)

        let mut final_scores_a: Vec<(String, f32, f32, f32)> = evaluations
            .iter()
            .map(|eval| {
                let ts = total_score(eval, importance_fluctuating, bpm_importance, brightness_importance, key_importance);
                let hb = history_bonus(&big_wave_mostly_played, &eval.track.id);
                let fs = final_score(ts, hb);
                (eval.track.id.clone(), ts, hb, fs)
            })
            .collect();
        final_scores_a.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

        println!("--- final_score (検証A: track_1〜15既再生、track_16〜21未再生) ---");
        for (id, ts, hb, fs) in &final_scores_a {
            let played = (1..=15).any(|i| format!("track_{}", i) == *id);
            println!("{}: total={:.4}, history_bonus={:.4}, final={:.4} [{}]", id, ts, hb, fs, if played { "既再生" } else { "未再生" });
        }

        // --- shortlistの動作確認: 検証Aと同じ条件でトップ7を絞り込む ---
        let top7 = shortlist(
            &evaluations,
            &big_wave_mostly_played,
            importance_fluctuating,
            bpm_importance,
            brightness_importance,
            key_importance,
            7,
        );
        println!("--- shortlist (top 7) ---");
        for track in &top7 {
            println!("{}", track.id);
        }

        // select_nextの動作確認: top7から10回選んでみて分布を確認
        println!("--- select_next (10 trials) ---");
        for _ in 0..10 {
            if let Some(track) = select_next(&top7) {
                println!("{}", track.id);
            }
        }
    }
}