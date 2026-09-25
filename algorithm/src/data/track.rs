//! アルゴリズムに使うそのままの形に型を定義する。
//!
//! Trackはmmz_format::TrackSummary(MyMusicAnalyzer側で事前計算された
//! 軽量なサマリ)から直接構築される。AnalysisOutput(生フレームデータ)
//! を扱う経路はここでは持たない。

use mmz_format::TrackSummary;

pub type TrackId = String;

pub struct KeySummary {
    pub name: String,
    pub correlation: f32,
}

pub struct SectionSummary {
    pub start_frame: usize,
    pub end_frame: usize,
    pub section_type: String,
}

pub struct EnergySummary {
    pub overall: f32,
    pub intro: f32,
    pub outro: f32,
}

pub struct BrightnessSummary {
    pub overall: f32,
    pub intro: f32,
    pub outro: f32,
}

pub struct Track {
    pub id: TrackId,
    pub bpm: f32,
    pub key: KeySummary,
    pub energy_summary: EnergySummary,
    pub brightness_summary: BrightnessSummary,
    pub sections: Vec<SectionSummary>,
}

impl Track {
    pub fn from_summary(id: TrackId, summary: TrackSummary) -> Self {
        Track {
            id,
            bpm: summary.bpm,
            key: KeySummary {
                name: summary.key.name,
                correlation: summary.key.correlation,
            },
            energy_summary: EnergySummary {
                overall: summary.energy.overall,
                intro: summary.energy.intro,
                outro: summary.energy.outro,
            },
            brightness_summary: BrightnessSummary {
                overall: summary.brightness.overall,
                intro: summary.brightness.intro,
                outro: summary.brightness.outro,
            },
            sections: summary
                .sections
                .into_iter()
                .map(|s| SectionSummary {
                    start_frame: s.start_frame,
                    end_frame: s.end_frame,
                    section_type: s.section_type,
                })
                .collect(),
        }
    }
}