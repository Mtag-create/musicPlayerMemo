//! JNI境界モジュール
//!
//! Kotlin側(RustBridge.kt)からのJNI呼び出しを受け付け、
//! MessagePackのエンコード/デコードのみを担当する。
//! 選曲ロジック本体(selectionモジュール)・状態管理(stateモジュール)には
//! 直接関与せず、それぞれの公開APIを呼び出すだけに留める。

use jni::errors::{Result as JniResult, ThrowRuntimeExAndDefault};
use jni::objects::{JByteArray, JClass};
use jni::{Env, EnvUnowned};
use serde::{de::DeserializeOwned, Serialize};

use crate::data::import::load_track;
use crate::data::library::Library;
use crate::protocol::{InitLibraryRequest, InitLibraryResponse};
use crate::protocol::{SelectNextRequest, SelectNextResponse};
use crate::protocol::SetCurrentTrackRequest;
use crate::selection::selector::select_random_from_library;
use crate::state;

use crate::selection::candidate::generate_candidates;
use crate::selection::flow::trend::{
    analyze_bpm_trend, analyze_brightness_trend, analyze_energy_trend, analyze_key_stability,
    bpm_axis_importance, brightness_axis_importance, energy_axis_importance, key_axis_importance,
};
use crate::selection::scoring::evaluate_candidates;
use crate::selection::selector::select_next;
use crate::selection::shortlist::shortlist;

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut serializer = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    value
        .serialize(&mut serializer)
        .expect("serialization should not fail for well-formed types");
    buf
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}

/// ライブラリ初期化: 全曲分の.mmzバイト列を受け取り、AppStateを構築する。
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mymusicplayer_rust_RustBridge_initLibrary<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JByteArray<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env: &mut Env<'local>| -> JniResult<JByteArray<'local>> {
            let bytes = env.convert_byte_array(&input)?;

            let response = match decode::<InitLibraryRequest>(&bytes) {
                Ok(request) => {
                    let mut tracks = Vec::new();
                    let mut failed_ids = Vec::new();

                    for entry in request.files {
                        match load_track(entry.id.clone(), &entry.mmz_bytes) {
                            Ok(track) => tracks.push(track),
                            Err(_) => failed_ids.push(entry.id),
                        }
                    }

                    let track_count = tracks.len();
                    let library = Library::new(tracks);
                    let success = state::replace_library(library);

                    InitLibraryResponse {
                        success,
                        track_count,
                        failed_ids,
                    }
                }
                Err(_) => InitLibraryResponse {
                    success: false,
                    track_count: 0,
                    failed_ids: Vec::new(),
                },
            };

            let encoded = encode(&response);
            env.byte_array_from_slice(&encoded)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

/// selectNextの戻り値に載せる、選ばれた曲のサマリ値。
struct SelectedTrackInfo {
    id: String,
    energy: f32,
    bpm: f32,
    key_name: String,
    key_correlation: f32,
    brightness: f32,
    energy_importance: Option<f32>,
    bpm_importance: Option<f32>,
    brightness_importance: Option<f32>,
    key_importance: Option<f32>,
}

/// 次曲選択。
/// 現在の曲(AppState.current_track_id)がなければランダムに1曲(初回選曲)、
/// あれば候補生成→スコアリング(4軸の動的重要度)→履歴補正→約7曲への絞り込み
/// →ランダム選択、という一連のパイプラインを実行する。
/// 選ばれた曲は即座にShortWave/BigWave/current_track_idへ反映される。
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mymusicplayer_rust_RustBridge_selectNext<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JByteArray<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env: &mut Env<'local>| -> JniResult<JByteArray<'local>> {
            let bytes = env.convert_byte_array(&input)?;
            let request: SelectNextRequest = decode(&bytes).unwrap_or(SelectNextRequest {
                energy_multiplier: 1.0,
                bpm_multiplier: 1.0,
                key_multiplier: 1.0,
                brightness_multiplier: 1.0,
            });
            let selected_info = state::with_state(|app_state| {
                let (selected, importance): (Option<String>, Option<(f32, f32, f32, f32)>) =
                    match app_state.current_track_id.clone() {
                        None => (select_random_from_library(&app_state.library), None),
                        Some(current_id) => {
                            let current_track = app_state
                                .library
                                .tracks()
                                .iter()
                                .find(|t| t.id == current_id);

                            match current_track {
                                None => (select_random_from_library(&app_state.library), None),
                                Some(current_track) => {
                                    let candidates =
                                        generate_candidates(&app_state.library, Some(&current_id));
                                    let evaluations = evaluate_candidates(
                                        &candidates,
                                        current_track,
                                        &app_state.energy_curve,
                                        app_state.big_wave.current_position(),
                                    );

                                    let energy_importance = energy_axis_importance(
                                        analyze_energy_trend(&app_state.short_wave, &app_state.library),
                                    ) * request.energy_multiplier;
                                    let bpm_importance = bpm_axis_importance(
                                        analyze_bpm_trend(&app_state.short_wave, &app_state.library),
                                    ) * request.bpm_multiplier;
                                    let brightness_importance = brightness_axis_importance(
                                        analyze_brightness_trend(&app_state.short_wave, &app_state.library),
                                    ) * request.brightness_multiplier;
                                    let key_importance = key_axis_importance(
                                        analyze_key_stability(&app_state.short_wave, &app_state.library),
                                    ) * request.key_multiplier;

                                    let top7 = shortlist(
                                        &evaluations,
                                        &app_state.big_wave,
                                        energy_importance,
                                        bpm_importance,
                                        brightness_importance,
                                        key_importance,
                                        7,
                                    );

                                    let selected_id = select_next(&top7).map(|t| t.id.clone());
                                    (
                                        selected_id,
                                        Some((energy_importance, bpm_importance, brightness_importance, key_importance)),
                                    )
                                }
                            }
                        }
                    };

                if let Some(ref id) = selected {
                    app_state.short_wave.push(id.clone());
                    app_state.big_wave.push(id.clone());
                    app_state.current_track_id = Some(id.clone());
                }

                selected.and_then(|id| {
                    app_state
                        .library
                        .tracks()
                        .iter()
                        .find(|t| t.id == id)
                        .map(|t| SelectedTrackInfo {
                            id: t.id.clone(),
                            energy: t.energy_summary.overall,
                            bpm: t.bpm,
                            key_name: t.key.name.clone(),
                            key_correlation: t.key.correlation,
                            brightness: t.brightness_summary.overall,
                            energy_importance: importance.map(|imp| imp.0),
                            bpm_importance: importance.map(|imp| imp.1),
                            brightness_importance: importance.map(|imp| imp.2),
                            key_importance: importance.map(|imp| imp.3),
                        })
                })
            })
            .flatten();

            let response = match selected_info {
                Some(info) => SelectNextResponse {
                    next_track_id: Some(info.id),
                    energy: Some(info.energy),
                    bpm: Some(info.bpm),
                    key_name: Some(info.key_name),
                    key_correlation: Some(info.key_correlation),
                    brightness: Some(info.brightness),
                    energy_importance: info.energy_importance,
                    bpm_importance: info.bpm_importance,
                    brightness_importance: info.brightness_importance,
                    key_importance: info.key_importance,
                },
                None => SelectNextResponse {
                    next_track_id: None,
                    energy: None,
                    bpm: None,
                    key_name: None,
                    key_correlation: None,
                    brightness: None,
                    energy_importance: None,
                    bpm_importance: None,
                    brightness_importance: None,
                    key_importance: None,
                },
            };
            let encoded = encode(&response);
            env.byte_array_from_slice(&encoded)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

/// 曲一覧画面などから、ユーザーが手動で選んだ曲を「現在の曲」として設定する。
/// selectNextと違ってスコアリングは一切行わず、指定されたtrack_idをそのまま反映するだけ。
/// ただし選ばれた曲はShortWave/BigWaveにも反映されるため、
/// 以降のAuto DJの選曲(トレンド分析)にはこの曲も履歴として影響する。
/// レスポンスはSelectNextResponseと同じ形を使い回す(importance系は常にNone)。
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mymusicplayer_rust_RustBridge_setCurrentTrack<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    input: JByteArray<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env: &mut Env<'local>| -> JniResult<JByteArray<'local>> {
            let bytes = env.convert_byte_array(&input)?;
            let request: SetCurrentTrackRequest = decode(&bytes).unwrap_or(SetCurrentTrackRequest {
                track_id: String::new(),
            });

            let selected_info = state::with_state(|app_state| {
                let found = app_state
                    .library
                    .tracks()
                    .iter()
                    .find(|t| t.id == request.track_id)
                    .map(|t| SelectedTrackInfo {
                        id: t.id.clone(),
                        energy: t.energy_summary.overall,
                        bpm: t.bpm,
                        key_name: t.key.name.clone(),
                        key_correlation: t.key.correlation,
                        brightness: t.brightness_summary.overall,
                        energy_importance: None,
                        bpm_importance: None,
                        brightness_importance: None,
                        key_importance: None,
                    });

                if let Some(ref info) = found {
                    app_state.short_wave.push(info.id.clone());
                    app_state.big_wave.push(info.id.clone());
                    app_state.current_track_id = Some(info.id.clone());
                }

                found
            })
            .flatten();

            let response = match selected_info {
                Some(info) => SelectNextResponse {
                    next_track_id: Some(info.id),
                    energy: Some(info.energy),
                    bpm: Some(info.bpm),
                    key_name: Some(info.key_name),
                    key_correlation: Some(info.key_correlation),
                    brightness: Some(info.brightness),
                    energy_importance: None,
                    bpm_importance: None,
                    brightness_importance: None,
                    key_importance: None,
                },
                None => SelectNextResponse {
                    next_track_id: None,
                    energy: None,
                    bpm: None,
                    key_name: None,
                    key_correlation: None,
                    brightness: None,
                    energy_importance: None,
                    bpm_importance: None,
                    brightness_importance: None,
                    key_importance: None,
                },
            };
            let encoded = encode(&response);
            env.byte_array_from_slice(&encoded)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}