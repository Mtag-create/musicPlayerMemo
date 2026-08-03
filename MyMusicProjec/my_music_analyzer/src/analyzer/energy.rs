use crate::analyzer::features::RmsFeature;
use crate::analyzer::band_energy::BandEnergy;

#[derive(Debug)]
pub struct EnergyFrame {
    pub index: usize,
    pub rms: f32,
    pub low: f32,
    pub mid: f32,
    pub high: f32,
}

pub fn merge_energy(
    rms: &RmsFeature,
    band: &BandEnergy,
) -> Vec<EnergyFrame> {

    let length =
        rms.frame_rms.len()
        .min(band.low.len())
        .min(band.mid.len())
        .min(band.high.len());


    let mut frames = Vec::with_capacity(length);


    for index in 0..length {

        frames.push(
            EnergyFrame {
                index,
                rms: rms.frame_rms[index],
                low: band.low[index],
                mid: band.mid[index],
                high: band.high[index],
            }
        );
    }


    frames
}