//! Earlier methods such as those of Lyderson (1955), Ambrose (1978; 1979;
//! 1980), and Fedors (1982) are described in previous editions; they do not
//! appear to be as accurate as those evaluated here.

use crate::r#const::atoms::C;
use molecule::Counter;
use uom::si::{f64::ThermodynamicTemperature, thermodynamic_temperature::kelvin};

/// Critical temperature
pub trait CriticalTemperature {
    fn critical_temperature(&self) -> ThermodynamicTemperature;
}

impl CriticalTemperature for Counter {
    fn critical_temperature(&self) -> ThermodynamicTemperature {
        // [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
        // [Fisher](https://sci-hub.ru/https://doi.org/10.1007/BF02670103)
        let n_c = self.count(C) as f64;
        let t_c = (n_c + 13.4) / (0.024984 + 0.00072433 * n_c);
        ThermodynamicTemperature::new::<kelvin>(t_c)
    }
}
