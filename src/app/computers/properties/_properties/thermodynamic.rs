use std::{
    cmp::{max, min},
    f64::consts::LN_2,
};

use molecule::{Counter, Saturable};
use uom::si::{
    f64::ThermodynamicTemperature,
    thermodynamic_temperature::{degree_celsius, kelvin},
};

use crate::{
    acylglycerol::Tag,
    r#const::{atoms::C, R},
};

/// Thermodynamic properties
///
/// The group contribution method GC-method is based on the fact that a
/// triglyceride can be defined as consisting of one median acyl group and two
/// terminal acyl groups. The GC-method is constructed, so that the group
/// interaction parameters describe next to which terminal acyl groups the
/// median acyl group is located. Thus, the GC-method takes into account the
/// position and the individual size of the acyl groups in the triglycerides.
/// The construction of the model is shown in Fig. 2. It has not been considered
/// how the acyl groups were placed in the real crystal structures when the
/// GC-method was constructed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Thermodynamic {
    pub alpha: Properties,
    pub beta_prime: Properties,
    pub beta: Properties,
}

impl Thermodynamic {
    pub(super) fn new(formula: Tag<&Counter>) -> Self {
        Self {
            alpha: Properties::new(formula, Polymorphism::Alpha),
            beta_prime: Properties::new(formula, Polymorphism::Beta(false)),
            beta: Properties::new(formula, Polymorphism::Beta(true)),
        }
    }

    pub(super) fn properties(&self, polymorphism: Polymorphism) -> &Properties {
        match polymorphism {
            Polymorphism::Alpha => &self.alpha,
            Polymorphism::Beta(false) => &self.beta_prime,
            Polymorphism::Beta(true) => &self.beta,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Polymorphism {
    Alpha,
    Beta(bool),
}

// amarango@uoguelph.ca
// Alejandro G. Marangoni, Ph.D.

// Я пытаюсь воспроизвести рассчеты, которые производит ваш калькулятор (https://lipidlibrary.shinyapps.io/Triglyceride_Property_Calculator/).
// Я столкнулся с проблемой, что расчитываемые значения энтропии не совпадают со значениями, выводимыми калькулятором.

// Пример:

#[derive(Clone, Copy, Debug, Default)]
pub struct Properties {
    // Enthalpy of fusion (ΔH)
    pub enthalpy_of_fusion: f64,
    /// Entropy of fusion (ΔS)
    pub entropy_of_fusion: f64,
    // The melting point of a TAG
    pub melting_points: (ThermodynamicTemperature, ThermodynamicTemperature),
}

impl Properties {
    // pub(super) fn alpha(formula: Tag<&Counter>) -> Self {
    //     Self::new(formula, Constants::ALPHA)
    // }

    // pub(super) fn beta(formula: Tag<&Counter>, prime: bool) -> Self {
    //     if prime {
    //         Self::new(formula, Constants::BETA_PRIME)
    //     } else {
    //         Self::new(formula, Constants::BETA)
    //     }
    // }

    fn new(formula: Tag<&Counter>, polymorphism: Polymorphism) -> Self {
        let Constants {
            h,
            h0,
            h_xy,
            h_odd,
            s,
            s0,
            s_xy,
            k,
            x0,

            t_inf,
            a_0,
            a_x,
            a_xx,
            a_xy,
            a_y,
            a_yy,
            b_0,
            b_x,
            b_xx,
            b_xy,
            b_y,
            b_yy,

            h_o,
            h_e,
            h_l: h_i,
        } = match polymorphism {
            Polymorphism::Alpha => Constants::ALPHA,
            Polymorphism::Beta(false) => Constants::BETA_PRIME,
            Polymorphism::Beta(true) => Constants::BETA,
        };

        let n1 = formula[0].count(C);
        let n2 = formula[1].count(C);
        let n3 = formula[2].count(C);
        let n = n1 + n2 + n3;
        let u1 = formula[0].unsaturated();
        let u2 = formula[1].unsaturated();
        let u3 = formula[2].unsaturated();
        let u = u1 + u2 + u3;
        let p = min(n1, n3);
        let q = n2;
        let r = max(n1, n3);
        let x = q as isize - p as isize;
        let y = r as isize - p as isize;

        let f_asym = if n1 != n3 || u1 != u3 { 1.0 } else { 0.0 };
        let f_odd = if n1 % 2 == 1 || n2 % 2 == 1 || n3 % 2 == 1 {
            1.0
        } else {
            0.0
        };
        let s_odd = 0.0;
        let f_b = match polymorphism {
            Polymorphism::Alpha => 0.0,
            Polymorphism::Beta(_) => 1.0,
        };

        let n = n as f64;
        let x = x as f64;
        let y = y as f64;
        let f_xy = 2.0 - (-((x - x0) / k).powi(2)).exp() - (-(y / k).powi(2)).exp();
        // let f_xy = 2.0 - (-((x - x0) / k).powi(2)).exp() - ((y / k).powi(2)).exp();

        // saturated TAGs
        let _h = h0 + h_xy * f_xy + h_odd * f_odd * f_b;
        let _s = s0 + s_xy * f_xy + s_odd * f_odd * f_b + R * LN_2 * f_asym * f_b;
        let dh = h * n + _h;
        let ds = s * n + _s;
        let t0 = {
            let b = _s / s;
            let a = _h / h - b;
            let t_inf = h / s;
            t_inf * (1.0 + a / n - a * b / n.powi(2))
        };
        let t1 = {
            let b = b_0 + b_x * x + b_xx * x.powi(2) + b_xy * x * y + b_y * y + b_yy * y.powi(2);
            let a = a_0 + a_x * x + a_xx * x.powi(2) + a_xy * x * y + a_y * y + a_yy * y.powi(2);
            t_inf * (1.0 + a / n - a * b / n.powi(2))
        };
        tracing::error!(?polymorphism, t0, t1);
        // unsaturated TAGs
        // let n_o = formula
        //     .map(|counter| if counter == &OLEIC { 1.0 } else { 0.0 })
        //     .sum();
        // let dh = dh + h_o * n_o + h_o * n_e + h_e * n_o;

        // Palmitic-Palmitic-Palmitic
        // n1=16 n2=16 n3=16 n=48.0 u1=0 u2=0 u3=0 u=0 p=16 q=16 r=16 x=0.0 y=0.0
        // polymorphism=Alpha:
        // f_xy=0.07787616392032737 f_asym=0.0 f_b=0.0 f_odd=0.0 s_odd=0.0
        // polymorphism=Beta(false):
        // f_xy=0.7830619019775951 f_asym=0.0 f_b=1.0 f_odd=0.0 s_odd=0.0
        // polymorphism=Beta(true):
        // f_xy=0.06898693055772842 f_asym=0.0 f_b=1.0 f_odd=0.0 s_odd=0.0
        // tracing::error!(?polymorphism);
        // tracing::error!(n1, n2, n3, n, u1, u2, u3, u, p, q, r, x, y);
        // tracing::error!(f_xy, asym, f_b, odd, s_odd);
        // let s2 = s1 / s;
        // let h2 = h1 / h;
        // let t1 = h / s * (1.0 + (h2 - s2) / n - s2 * (h2 - s2) / n.powi(2));
        Properties {
            enthalpy_of_fusion: dh,
            entropy_of_fusion: ds,
            melting_points: (
                ThermodynamicTemperature::new::<kelvin>(1000.0 * t0),
                ThermodynamicTemperature::new::<kelvin>(t1),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Constants {
    // Saturated
    h: f64,
    h0: f64,
    h_xy: f64,
    h_odd: f64,
    s: f64,
    s0: f64,
    s_xy: f64,
    k: f64,
    x0: f64,

    t_inf: f64,
    // A
    a_0: f64,
    a_x: f64,
    a_xx: f64,
    a_xy: f64,
    a_y: f64,
    a_yy: f64,
    // B
    b_0: f64,
    b_x: f64,
    b_xx: f64,
    b_xy: f64,
    b_y: f64,
    b_yy: f64,
    // Unsaturated
    h_o: f64,
    h_e: f64,
    h_l: f64,
}

// h0 s0 - are the contributions due to the glycerol head group,
impl Constants {
    pub const ALPHA: Self = Self {
        h: 2.70,
        s: 6.79,

        h0: -31.95,
        s0: -19.09,

        h_xy: -13.28,
        s_xy: -36.70,

        x0: 1.25,
        k: 4.39,
        h_odd: 0.0,

        t_inf: 401.15,
        a_0: -9.0581,
        a_x: 0.00290,
        a_xx: -0.0619116,
        a_xy: 0.115128,
        a_y: -0.453461,
        a_yy: -0.005827,
        b_0: -4.4841,
        b_x: -0.00111,
        b_xx: 0.148938,
        b_xy: -0.365917,
        b_y: 1.41154,
        b_yy: -0.001766,

        h_o: -31.7,
        h_e: -11.7,
        h_l: -37.7,
    };

    pub const BETA_PRIME: Self = Self {
        h: 3.86,
        s: 10.13,

        h0: -35.86,
        s0: -39.59,

        h_xy: -19.35,
        s_xy: -52.51,

        x0: 2.46,
        k: 1.99,
        h_odd: 0.0,

        t_inf: 401.15,
        a_0: -8.4543,
        a_x: -0.10360,
        a_xx: -0.018881,
        a_xy: 0.0739411,
        a_y: -0.49721,
        a_yy: 0.0115995,
        b_0: -0.26501,
        b_x: 0.54997,
        b_xx: 0.074136,
        b_xy: -0.340928,
        b_y: 2.34238,
        b_yy: -0.135735,

        h_o: -28.3,
        h_e: -15.9,
        h_l: -37.7,
    };

    pub const BETA: Self = Self {
        h: 3.89,
        s: 9.83,

        h0: -17.16,
        s0: 31.04,

        h_xy: -22.29,
        s_xy: -64.58,

        x0: 0.77,
        k: 2.88,
        h_odd: 2.29,

        t_inf: 401.15,
        a_0: -8.0481,
        a_x: 0.074130,
        a_xx: -0.0348596,
        a_xy: 0.00771420,
        a_y: -0.404136,
        a_yy: 0.0111938,
        b_0: 2.66923,
        b_x: -0.31675,
        b_xx: 0.085967,
        b_xy: 0.040642,
        b_y: 0.55040,
        b_yy: -0.000945,

        h_o: -30.2,
        h_e: -15.9,
        h_l: -37.7,
    };
}
