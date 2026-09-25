#[derive(Debug)]
pub struct RawAudio {
    /// 正規化済みPCMデータ（-1.0 ～ 1.0）
    pub samples: Vec<f32>,

    /// サンプルレート
    pub sample_rate: u32,

    /// チャンネル数
    pub channels: u16,
}