use crate::analyzer::features::SpectrumFeature;


#[derive(Debug)]
pub struct SpectralFeature {
    pub centroid: Vec<f32>,
    pub flux: Vec<f32>,
    pub rolloff: Vec<f32>,
}


pub fn calculate_spectral_features(
    spectra: &Vec<SpectrumFeature>,
) -> SpectralFeature {

    let mut centroid = Vec::new();
    let mut flux = Vec::new();
    let mut rolloff = Vec::new();


    for i in 0..spectra.len() {

        let spectrum = &spectra[i];


        // Centroid
        let mut numerator = 0.0;
        let mut denominator = 0.0;


        for (freq, mag) in spectrum
            .frequencies
            .iter()
            .zip(spectrum.magnitudes.iter())
        {
            numerator += freq * mag;
            denominator += mag;
        }


        if denominator > 0.0 {
            centroid.push(
                numerator / denominator
            );
        } else {
            centroid.push(0.0);
        }


        // Flux
        if i == 0 {
            flux.push(0.0);
        } else {

            let previous = &spectra[i-1];

            let mut sum = 0.0;

            for (a,b) in previous
                .magnitudes
                .iter()
                .zip(spectrum.magnitudes.iter())
            {
                let diff = b - a;

                if diff > 0.0 {
                    sum += diff * diff;
                }
            }

            flux.push(sum.sqrt());
        }


        // Roll-off
        let total: f32 =
            spectrum.magnitudes
            .iter()
            .map(|x| x*x)
            .sum();


        let target = total * 0.85;


        let mut accum = 0.0;
        let mut freq_value = 0.0;


        for (freq, mag) in spectrum
            .frequencies
            .iter()
            .zip(spectrum.magnitudes.iter())
        {
            accum += mag * mag;

            if accum >= target {
                freq_value = *freq;
                break;
            }
        }


        rolloff.push(freq_value);
    }


    SpectralFeature {
        centroid,
        flux,
        rolloff,
    }
}