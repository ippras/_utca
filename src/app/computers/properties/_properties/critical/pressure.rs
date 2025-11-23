//! The group contribution method by Joback is recommended for estimation of the
//! critical pressure

use crate::r#const::atoms::C;
use molecule::Counter;
use uom::si::{f64::Pressure, pressure::pascal};

/// Critical pressure
pub trait CriticalPressure {
    fn critical_pressure(&self) -> Pressure;
}

impl CriticalPressure for Counter {
    fn critical_pressure(&self) -> Pressure {
        // [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
        // [Fisher](https://sci-hub.ru/https://doi.org/10.1007/BF02670103)
        let n_c = self.count(C) as f64;
        let p_c = (n_c + 13.4) / (0.024984 + 0.00072433 * n_c);
        Pressure::new::<pascal>(p_c)
    }
}
