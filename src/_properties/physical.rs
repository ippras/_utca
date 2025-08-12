use crate::{acylglycerol::Tag, r#const::atoms::C};
use molecule::{Counter, Saturable};
use uom::si::{
    dynamic_viscosity::pascal_second,
    f64::{DynamicViscosity, MolarVolume, ThermodynamicTemperature},
    molar_volume::cubic_meter_per_mole,
    thermodynamic_temperature::kelvin,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Properties {
    pub(crate) dynamic_viscosity: DynamicViscosity,
    pub(crate) molar_volume: MolarVolume,
    pub(crate) critical_temperatures: Tag<ThermodynamicTemperature>,
}

impl Properties {
    pub(crate) fn fatty_acid(formula: &Counter, t: f64) -> Self {
        const A1: f64 = -6.09;
        const A2: f64 = -3.536;
        const A3: f64 = 5.40;
        const A4: f64 = 3.10;
        const A5: f64 = -0.066;
        const B1: f64 = 1331.5;
        const C1: f64 = 41.6;
        const C2: f64 = 4.135;
        const C3: f64 = -8.0;

        let nc = formula.count(C) as f64;
        let nd = formula.unsaturated() as f64;
        let a = (A1 - A2) / (1.0 + ((nc - A3) / A4).exp()) + A2 + A5 * nd.powi(2);
        let b = B1;
        let c = C1 + C2 * nc + C3 * nd;
        let n = a + b / (t - c);
        Self {
            dynamic_viscosity: DynamicViscosity::new::<pascal_second>(n),
            ..Default::default()
        }
    }

    pub(crate) fn triglyceride(formula: Tag<&Counter>, t: f64) -> Self {
        const A1: f64 = -4.01;
        const A2: f64 = -2.954;
        const A3: f64 = 28.9;
        const A4: f64 = 6.5;
        const A5: f64 = -0.0033;
        const B1: f64 = 1156.0;
        const C1: f64 = 99.1;
        const C2: f64 = 0.851;
        const C3: f64 = -3.65;

        let nc = formula.map(|counter| counter.count(C)).sum() as f64; // TODO + gli
        let nd = formula.map(|counter| counter.unsaturated()).sum() as f64;

        let dynamic_viscosity = {
            let a = (A1 - A2) / (1.0 + ((nc - A3) / A4).exp()) + A2 + A5 * nd.powi(2);
            let b = B1;
            let c = C1 + C2 * nc + C3 * nd;
            let n = a + b / (t - c);
            DynamicViscosity::new::<pascal_second>(n)
        };

        // [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
        // [Hammond and Lundberg](https://sci-hub.ru/10.1007/BF02639027)
        let molar_volume = {
            let unsaturated = formula
                .map(|saturable| !saturable.saturated() as usize)
                .sum();
            let v_m = match unsaturated {
                0..=1 => 16.54 * nc - 6.65 * nd + 26.09 + (0.006 * nc + 0.0085) * (t - 20.0),
                // polyunsaturated
                _ => 16.54 * nc - 6.87 * nd + 26.09 + (0.006 * nc + 0.0085) * (t - 20.0),
            };
            MolarVolume::new::<cubic_meter_per_mole>(v_m)
        };
        // [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
        let critical_temperatures = formula.map(|counter| {
            let c = counter.count(C) as f64;
            let t_c = (c + 13.4) / (0.024984 + 0.00072433 * c);
            ThermodynamicTemperature::new::<kelvin>(t_c)
        });
        Self {
            dynamic_viscosity,
            molar_volume,
            critical_temperatures,
        }
    }
}
