use crate::r#const::atoms::C;
use molecule::{Counter, Saturable};
use uom::si::{
    dynamic_viscosity::{centipoise, millipascal_second, pascal_second},
    f64::{DynamicViscosity, ThermodynamicTemperature},
    thermodynamic_temperature::{degree_celsius, kelvin},
};

pub enum Method {
    Rabelo,
}

/// Viscosity
pub trait Viscosity {
    fn dynamic_viscosity(&self, method: Method) -> DynamicViscosity;
}

/// Rabelo method for estimating fatty acid dynamic viscosity
/// [Rabelo](https://sci-hub.ru/10.1007/s11746-000-0197-z)
pub trait Rabelo {
    fn dynamic_viscosity(&self, temperature: ThermodynamicTemperature) -> DynamicViscosity;
}

impl Rabelo for Counter {
    fn dynamic_viscosity(&self, temperature: ThermodynamicTemperature) -> DynamicViscosity {
        const A1: f64 = -6.09;
        const A2: f64 = -3.536;
        const A3: f64 = 5.40;
        const A4: f64 = 3.10;
        const A5: f64 = -0.066;
        const B1: f64 = 1331.5;
        const C1: f64 = 41.6;
        const C2: f64 = 4.135;
        const C3: f64 = -8.0;

        let n_c = self.count(C) as f64;
        let n_d = self.unsaturated() as f64;
        let t = temperature.get::<kelvin>();
        let a = (A1 - A2) / (1.0 + ((n_c - A3) / A4).exp()) + A2 + A5 * n_d.powi(2);
        let b = B1;
        let c = C1 + C2 * n_c + C3 * n_d;
        let n = a + b / (t - c);
        // tracing::error!(n_c, n_d, t, n);
        DynamicViscosity::new::<millipascal_second>(n)
    }
}

// impl Rabelo for Tag<Counter> {
//     fn dynamic_viscosity(&self, temperature: ThermodynamicTemperature) -> DynamicViscosity {
//         const A1: f64 = -4.01;
//         const A2: f64 = -2.954;
//         const A3: f64 = 28.9;
//         const A4: f64 = 6.5;
//         const A5: f64 = -0.0033;
//         const B1: f64 = 1156.0;
//         const C1: f64 = 99.1;
//         const C2: f64 = 0.851;
//         const C3: f64 = -3.65;

//         let n_c = self.count(C) as f64;
//         let n_d = self.unsaturated() as f64;
//         let t = temperature.get::<kelvin>();
//         let a = (A1 - A2) / (1.0 + ((n_c - A3) / A4).exp()) + A2 + A5 * n_d.powi(2);
//         let b = B1;
//         let c = C1 + C2 * n_c + C3 * n_d;
//         let n = a + b / (t - c);
//         tracing::error!(n_c, n_d, t, n);
//         DynamicViscosity::new::<millipascal_second>(n)
//     }
// }
