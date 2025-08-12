use crate::r#const::atoms::C;
use molecule::{Counter, Saturable};
use uom::si::{
    dynamic_viscosity::{centipoise, millipascal_second, pascal_second},
    f64::{DynamicViscosity, Pressure, ThermodynamicTemperature},
    pressure::pascal,
    thermodynamic_temperature::{degree_celsius, kelvin},
};

/// Joback method for estimating fatty acid critical properties
/// [Joback](https://edisciplinas.usp.br/pluginfile.php/5516437/course/section/6014437/The%20Properties%20of%20Gases%20and%20Liquids%20%285th_ed%29-RC%20Reid%2C%20JM%20Prausnitz%20%20BE%20Poling%202004.pdf)
/// [Joback property functions fromgroup contributions](https://www.accessengineeringlibrary.com/content/book/9780070116825/back-matter/appendix3)
pub trait Joback {
    fn critical_pressure(&self) -> Pressure;

    /// Normal boiling point
    fn critical_temperature(
        &self,
        t_b: Option<ThermodynamicTemperature>,
    ) -> ThermodynamicTemperature;
}

// normal boiling point
impl Joback for Counter {
    fn critical_pressure(&self) -> Pressure {
        let n = self.count(C) as f64;
        // let p_c = (0.113 + 0.0032 * n - ).powi(-2);
        // Pressure::new::<pascal>(p_c)
        unimplemented!()
    }

    fn critical_temperature(
        &self,
        t_b: Option<ThermodynamicTemperature>,
    ) -> ThermodynamicTemperature {
        // [Halvorsen](https://sci-hub.ru/https://doi.org/10.1007/BF02545346)
        // [Fisher](https://sci-hub.ru/https://doi.org/10.1007/BF02670103)
        let n_c = self.count(C) as f64;
        let t_c = (n_c + 13.4) / (0.024984 + 0.00072433 * n_c);
        ThermodynamicTemperature::new::<kelvin>(t_c)
    }
}
