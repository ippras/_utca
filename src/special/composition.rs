use self::{
    Composition::*,
    Stereospecificity::{NonStereospecific, Stereospecific},
};
use crate::text::Text;
use serde::{Deserialize, Serialize};

pub const COMPOSITIONS: [Composition; 12] =
    [SMC, SPC, SSC, TMC, TPC, TSC, MMC, MSC, NMC, NSC, UMC, USC];

// Mass composition, non-stereospecific, agregation
pub const MMC: Composition = Mass(NonStereospecific(Agregation));
// Mass composition, stereospecific
pub const MSC: Composition = Mass(Stereospecific);

// Equivalent carbon number composition, non-stereospecific, agregation
pub const NMC: Composition = EquivalentCarbonNumber(NonStereospecific(Agregation));
// Equivalent carbon number composition, stereospecific
pub const NSC: Composition = EquivalentCarbonNumber(Stereospecific);

// Species composition, non-stereospecific, permutation
pub const SMC: Composition = Species(NonStereospecific(Permutation { positional: false }));
// Species composition, non-stereospecific, permutation, positional
pub const SPC: Composition = Species(NonStereospecific(Permutation { positional: true }));
// Species composition, stereospecific
pub const SSC: Composition = Species(Stereospecific);

// Type composition, non-stereospecific, permutation
pub const TMC: Composition = Type(NonStereospecific(Permutation { positional: false }));
// Type composition, non-stereospecific, permutation, positional
pub const TPC: Composition = Type(NonStereospecific(Permutation { positional: true }));
// Type composition, stereospecific
pub const TSC: Composition = Type(Stereospecific);

// Unsaturation composition, non-stereospecific, agregation
pub const UMC: Composition = Unsaturation(NonStereospecific(Agregation));
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

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            MMC => "mass_nonstereospecific_composition",
            MSC => "mass_stereospecific_composition",
            NMC => "equivalent_carbon_number_nonstereospecific_composition",
            NSC => "equivalent_carbon_number_stereospecific_composition",
            SMC => "species_nonstereospecific_composition",
            SPC => "species_positionalspecific_composition",
            SSC => "species_stereospecific_composition",
            TMC => "type_nonstereospecific_composition",
            TPC => "type_positionalspecific_composition",
            TSC => "type_stereospecific_composition",
            UMC => "unsaturation_nonstereospecific_composition",
            USC => "unsaturation_stereospecific_composition",
        }
    }

    fn hover_text(&self) -> &'static str {
        match *self {
            MMC => "mass_nonstereospecific_composition.hover",
            MSC => "mass_stereospecific_composition.hover",
            NMC => "equivalent_carbon_number_nonstereospecific_composition.hover",
            NSC => "equivalent_carbon_number_stereospecific_composition.hover",
            SMC => "species_nonstereospecific_composition.hover",
            SPC => "species_positionalspecific_composition.hover",
            SSC => "species_stereospecific_composition.hover",
            TMC => "type_nonstereospecific_composition.hover",
            TPC => "type_positionalspecific_composition.hover",
            TSC => "type_stereospecific_composition.hover",
            UMC => "unsaturation_nonstereospecific_composition.hover",
            USC => "unsaturation_stereospecific_composition.hover",
        }
    }
}

impl Default for Composition {
    fn default() -> Self {
        Self::new()
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
