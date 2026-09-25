/// Hann窓の係数配列を生成する。
///
/// w(n) = 0.5 * (1.0 - cos(2*PI*n / (size - 1)))
pub fn generate_hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size];
    }

    (0..size)
        .map(|n| {
            let ratio = n as f32 / (size - 1) as f32;
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * ratio).cos())
        })
        .collect()
}

/// サンプル列を frame_size 幅・hop_size 間隔でスライドさせながら切り出し、
/// 各フレームにHann窓を適用して返す。
///
/// frame_size に満たない末尾の端数フレームは含めない。
pub fn extract_frames(
    samples: &[f32],
    frame_size: usize,
    hop_size: usize,
) -> Vec<Vec<f32>> {
    let window = generate_hann_window(frame_size);

    let mut frames = Vec::new();

    if samples.len() < frame_size {
        return frames;
    }

    let mut start = 0;

    while start + frame_size <= samples.len() {
        let windowed: Vec<f32> = samples[start..start + frame_size]
            .iter()
            .zip(window.iter())
            .map(|(sample, w)| sample * w)
            .collect();

        frames.push(windowed);

        start += hop_size;
    }

    frames
}

/// 単純移動平均による平滑化。
///
/// 各インデックス i について、前後 window フレームの範囲(端は切り詰め)で平均を取る。
pub fn moving_average(values: &Vec<f32>, window: usize) -> Vec<f32> {

    let mut result = Vec::new();

    for i in 0..values.len() {

        let start =
            if i >= window {
                i - window
            } else {
                0
            };

        let end =
            if i + window < values.len() {
                i + window
            } else {
                values.len() - 1
            };

        let mut sum = 0.0;
        let mut count = 0;

        for j in start..=end {
            sum += values[j];
            count += 1;
        }

        result.push(sum / count as f32);
    }

    result
}