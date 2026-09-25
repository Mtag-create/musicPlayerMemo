use crate::analyzer::features::SpectrumFeature;


#[derive(Debug)]
pub struct BandEnergy {
    pub low: Vec<f32>,
    pub mid: Vec<f32>,
    pub high: Vec<f32>,
}


pub fn calculate_band_energy(
    spectrum_frames: &Vec<SpectrumFeature>,
) -> BandEnergy {

    let mut low = Vec::new();
    let mut mid = Vec::new();
    let mut high = Vec::new();

    for spectrum in spectrum_frames {

        let mut low_sum = 0.0;
        let mut mid_sum = 0.0;
        let mut high_sum = 0.0;


        for (freq, magnitude) in spectrum
            .frequencies
            .iter()
            .zip(spectrum.magnitudes.iter())
        {

            if *freq < 250.0 {
                low_sum += magnitude * magnitude;

            } else if *freq < 2000.0 {
                mid_sum += magnitude * magnitude;

            } else {
                high_sum += magnitude * magnitude;
            }
        }


        low.push(low_sum.sqrt());
        mid.push(mid_sum.sqrt());
        high.push(high_sum.sqrt());
    }


    BandEnergy {
        low,
        mid,
        high,
    }
}