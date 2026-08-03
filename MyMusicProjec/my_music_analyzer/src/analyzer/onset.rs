use crate::analyzer::energy::EnergyFrame;
use crate::analyzer::features::OnsetFeature;
use crate::analyzer::spectral::SpectralFeature;

pub fn analyze_onset(
    energy: &Vec<EnergyFrame>,
    spectral: &SpectralFeature,
) -> OnsetFeature {


    let mut strength = Vec::new();

    // 先頭フレームは差分が取れないため 0.0 とし、
    // strength[i] が energy[i] と対応するようインデックスを揃える
    strength.push(0.0);

    for i in 1..energy.len() {


        let low_diff =
            (energy[i].low - energy[i-1].low)
            .max(0.0);


        let rms_diff =
            (energy[i].rms - energy[i-1].rms)
            .max(0.0);


        let flux =
            spectral.flux[i];


        let value =
            low_diff * 0.45
            +
            flux * 0.45
            +
            rms_diff * 0.10;


        strength.push(value);
    }


let smoothed =
        crate::analyzer::framing::moving_average(
            &strength,
            4
        );

    OnsetFeature {
        strength,
        smoothed,
    }
}