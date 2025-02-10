use self::{
    Composition::*,
    Stereospecificity::{NonStereospecific, Stereospecific},
};
use serde::{Deserialize, Serialize};

pub const COMPOSITIONS: [Composition; 12] =
    [MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC];

// Mass composition, non-stereospecific, agregation
pub const MNC: Composition = Mass(NonStereospecific(Agregation));
// Mass composition, stereospecific
pub const MSC: Composition = Mass(Stereospecific);

// Equivalent carbon number composition, non-stereospecific, agregation
pub const NNC: Composition = EquivalentCarbonNumber(NonStereospecific(Agregation));
// Equivalent carbon number composition, stereospecific
pub const NSC: Composition = EquivalentCarbonNumber(Stereospecific);

// Species composition, non-stereospecific, permutation
pub const SNC: Composition = Species(NonStereospecific(Permutation { positional: false }));
// Species composition, non-stereospecific, permutation, positional
pub const SPC: Composition = Species(NonStereospecific(Permutation { positional: true }));
// Species composition, stereospecific
pub const SSC: Composition = Species(Stereospecific);

// Type composition, non-stereospecific, permutation
pub const TNC: Composition = Type(NonStereospecific(Permutation { positional: false }));
// Type composition, non-stereospecific, permutation, positional
pub const TPC: Composition = Type(NonStereospecific(Permutation { positional: true }));
// Type composition, stereospecific
pub const TSC: Composition = Type(Stereospecific);

// Unsaturation composition, non-stereospecific, agregation
pub const UNC: Composition = Unsaturation(NonStereospecific(Agregation));
// Unsaturation composition, stereospecific
pub const USC: Composition = Unsaturation(Stereospecific);

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Composition {
    EquivalentCarbonNumber(Stereospecificity<Agregation>),
    Mass(Stereospecificity<Agregation>),
    Species(Stereospecificity<Permutation>),
    Type(Stereospecificity<Permutation>),
    Unsaturation(Stereospecificity<Agregation>),
}

impl Composition {
    pub fn new() -> Self {
        SSC
    }
}

// /// Numeric
// pub enum Numeric {
//     Agregation(Stereospecificity<Agregation>),
//     Permutation(Stereospecificity<Permutation>),
// }

/// Stereospecificity
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Stereospecificity<T> {
    Stereospecific,
    NonStereospecific(T),
}

/// Agregation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Agregation;

/// Permutation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Permutation {
    pub positional: bool,
}

// Composition
// 1. Ecn(Option<Agregation>)
// • EA
// 2. Mass(Option<Agregation>)
// • MA
// 3. Unsaturation(Option<Agregation>)
// • UA
// 4. Species(Permutation)
// • SC
// • PSC
// • SSC
// 5. Type(Permutation)
// • TC
// • PTC
// • STC

// type Permutation = Option<Stereospecificity>;
// Operation:
// Permutation {
//     stereospecificity: Option<Stereospecificity>,
// }
// * Agregation
