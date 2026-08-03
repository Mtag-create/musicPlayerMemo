#[derive(Debug)]
pub struct RmsFeature {
    /// 曲全体のRMS
    pub overall_rms: f32,

    /// フレームごとのRMS
    pub frame_rms: Vec<f32>,
}

#[derive(Debug)]
pub struct SpectrumFeature {
    pub frequencies: Vec<f32>,
    pub magnitudes: Vec<f32>,
}

#[derive(Debug)]
pub struct BeatFeature {
    pub bpm: f32,
    pub bpm_candidates: Vec<f32>,
    pub beats: Vec<usize>,
    pub beat_strength: Vec<f32>,
}

#[derive(Debug)]
pub struct OnsetFeature {
    pub strength: Vec<f32>,
    pub smoothed: Vec<f32>,
}

#[derive(Debug)]
pub struct ChromaFeature {
    pub chroma: Vec<[f32; 12]>,
    #[allow(dead_code)]
    pub chroma_detailed: Vec<[f32; 60]>,
}

#[derive(Debug)]
pub struct KickFeature {
    pub strength: Vec<f32>,
    pub bandpass_envelope: Vec<f32>,
}

#[derive(Debug)]
pub struct EnergyCurveFeature {
    pub rms: Vec<f32>,
    pub low: Vec<f32>,
    pub mid: Vec<f32>,
    pub high: Vec<f32>,
    pub overall: Vec<f32>,
    pub peaks: Vec<usize>,
    pub valleys: Vec<usize>,
    pub sections: Vec<SectionFeature>,
    pub delta_overall: Vec<f32>,
    pub delta_low: Vec<f32>,
}

#[derive(Debug)]
pub struct SectionFeature {
    pub label: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct KeyCandidate {
    pub name: String,
    pub correlation: f32,
}

#[derive(Debug)]
pub struct KeyEstimate {
    pub candidates: Vec<KeyCandidate>,
}

#[derive(Debug)]
pub struct SectionKey {
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub estimate: KeyEstimate,
}

#[derive(Debug)]
pub struct KeyFeature {
    pub overall: KeyEstimate,
    pub sections: Vec<SectionKey>,
}

#[derive(Debug)]
pub struct BrightnessFeature {
    pub values: Vec<f32>,
}