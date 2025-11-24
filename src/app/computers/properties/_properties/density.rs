use crate::r#const::atoms::C;
use molecule::{Counter, Saturable};
use uom::si::{
    dynamic_viscosity::pascal_second,
    f64::{MassDensity, MolarMass, MolarVolume, ThermodynamicTemperature},
    mass_density::kilogram_per_cubic_meter,
    molar_mass::{gram_per_mole, kilogram_per_mole},
    molar_volume::{cubic_centimeter_per_mole, cubic_meter_per_mole},
    thermodynamic_temperature::degree_celsius,
};

/// Hammond method for estimating fatty acid density
/// [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
/// [Hammond](https://sci-hub.ru/10.1007/BF02639027)
pub trait Hammond {
    fn density(&self, temperature: ThermodynamicTemperature) -> MassDensity;

    fn molar_volume(&self, temperature: ThermodynamicTemperature) -> MolarVolume;
}

impl Hammond for Counter {
    fn density(&self, temperature: ThermodynamicTemperature) -> MassDensity {
        let molar_volume = self.molar_volume(temperature);
        let molar_mass = MolarMass::new::<gram_per_mole>(self.weight());
        molar_mass / molar_volume
    }

    fn molar_volume(&self, temperature: ThermodynamicTemperature) -> MolarVolume {
        let t = temperature.get::<degree_celsius>();
        let n_c = self.count(C) as f64;
        let n_d = self.unsaturated() as f64;
        // let v_m = match self.unsaturated() {
        //     // Monounsaturated
        //     0..=1 => 16.54 * n_c - 6.65 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
        //     // Polyunsaturated
        //     _ => 16.54 * n_c - 6.87 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
        // };
        let v_m = match self.unsaturated() {
            // Monounsaturated
            0..=1 => 16.54 * n_c - 6.65 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
            // Polyunsaturated
            _ => 16.54 * n_c - 6.87 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
        };
        MolarVolume::new::<cubic_centimeter_per_mole>(v_m)
    }
}

// 16.54 * 12 - 6.65 * 0 + 47.99 = 246.47
// 200.1776 / 246.47 = 0.81217

// 16.54 * 12 + 41.34 = 239.82
// 200.1776 / 239.82 = 0.834699

// 16.54 * 12 - 6.65 * 0 + 26.09 + (0.006 * 12 + 0.0085) * (20 - 20.0) = 224.57
// 200.1776 / 224.57 = 0.8913

/// Modified Rackett method for estimating fatty acid density
/// [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
pub trait RackettModified {
    fn density(&self, temperature: ThermodynamicTemperature) -> MassDensity;

    fn molar_volume(&self, temperature: ThermodynamicTemperature) -> MolarVolume;
}

// impl RackettModified for Counter {
//     fn density(&self, temperature: ThermodynamicTemperature) -> MassDensity {
//         let molar_volume = self.molar_volume(temperature);
//         let molar_mass = MolarMass::new::<gram_per_mole>(self.weight());
//         molar_mass / molar_volume
//     }

//     fn molar_volume(&self, temperature: ThermodynamicTemperature) -> MolarVolume {
//         assert!(self.saturated());
//         let t = temperature.get::<degree_celsius>();
//         let n_c = self.count(C) as f64;
//         let v_s = match self.unsaturated() {
//             // Monounsaturated
//             0..=1 => 16.54 * n_c - 6.65 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
//             // Polyunsaturated
//             _ => 16.54 * n_c - 6.87 * n_d + 26.09 + (0.006 * n_c + 0.0085) * (t - 20.0),
//         };
//         MolarVolume::new::<cubic_centimeter_per_mole>(v_s)
//     }
// }

// Verschaffelt
