use std::fmt::{self, Display, Formatter};

use self::{
    Composition::*,
    Stereospecificity::{NonStereospecific, Stereospecific},
};
use crate::text::Text;
use serde::{Deserialize, Serialize};

pub const COMPOSITIONS: [Composition; 12] = [
    SPECIES_MONO,
    SPECIES_POSITIONAL,
    SPECIES_STEREO,
    TYPE_MONO,
    TYPE_POSITIONAL,
    TYPE_STEREO,
    MASS_MONO,
    MASS_STEREO,
    ECN_MONO,
    ECN_STEREO,
    UNSATURATION_MONO,
    UNSATURATION_STEREO,
];

// Mass composition, non-stereospecific, agregation
pub const MASS_MONO: Composition = Mass(NonStereospecific(Agregation));
// Mass composition, stereospecific
pub const MASS_STEREO: Composition = Mass(Stereospecific);

// Equivalent carbon number composition, non-stereospecific, agregation
pub const ECN_MONO: Composition = EquivalentCarbonNumber(NonStereospecific(Agregation));
// Equivalent carbon number composition, stereospecific
pub const ECN_STEREO: Composition = EquivalentCarbonNumber(Stereospecific);

// Species composition, non-stereospecific, permutation
pub const SPECIES_MONO: Composition = Species(NonStereospecific(Permutation { positional: false }));
// Species composition, non-stereospecific, permutation, positional
pub const SPECIES_POSITIONAL: Composition =
    Species(NonStereospecific(Permutation { positional: true }));
// Species composition, stereospecific
pub const SPECIES_STEREO: Composition = Species(Stereospecific);

// Type composition, non-stereospecific, permutation
pub const TYPE_MONO: Composition = Type(NonStereospecific(Permutation { positional: false }));
// Type composition, non-stereospecific, permutation, positional
pub const TYPE_POSITIONAL: Composition = Type(NonStereospecific(Permutation { positional: true }));
// Type composition, stereospecific
pub const TYPE_STEREO: Composition = Type(Stereospecific);

// Unsaturation composition, non-stereospecific, agregation
pub const UNSATURATION_MONO: Composition = Unsaturation(NonStereospecific(Agregation));
// Unsaturation composition, stereospecific
pub const UNSATURATION_STEREO: Composition = Unsaturation(Stereospecific);

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
        SPECIES_STEREO
    }
}

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            MASS_MONO => "Composition-Mass-Monospecific",
            MASS_STEREO => "Composition-Mass-Stereospecific",
            ECN_MONO => "Composition-EquivalentCarbonNumber-Monospecific",
            ECN_STEREO => "Composition-EquivalentCarbonNumber-Stereospecific",
            SPECIES_MONO => "Composition-Species-Monospecific",
            SPECIES_POSITIONAL => "Composition-Species-Positionalspecific",
            SPECIES_STEREO => "Composition-Species-Stereospecific",
            TYPE_MONO => "Composition-Type-Monospecific",
            TYPE_POSITIONAL => "Composition-Type-Positionalspecific",
            TYPE_STEREO => "Composition-Type-Stereospecific",
            UNSATURATION_MONO => "Composition-Unsaturation-Monospecific",
            UNSATURATION_STEREO => "Composition-Unsaturation-Stereospecific",
        }
    }

    fn hover_text(&self) -> &'static str {
        match *self {
            MASS_MONO => "Composition-Mass-Monospecific.hover",
            MASS_STEREO => "Composition-Mass-Stereospecific.hover",
            ECN_MONO => "Composition-EquivalentCarbonNumber-Monospecific.hover",
            ECN_STEREO => "Composition-EquivalentCarbonNumber-Stereospecific.hover",
            SPECIES_MONO => "Composition-Species-Monospecific.hover",
            SPECIES_POSITIONAL => "Composition-Species-Positionalspecific.hover",
            SPECIES_STEREO => "Composition-Species-Stereospecific.hover",
            TYPE_MONO => "Composition-Type-Monospecific.hover",
            TYPE_POSITIONAL => "Composition-Type-Positionalspecific.hover",
            TYPE_STEREO => "Composition-Type-Stereospecific.hover",
            UNSATURATION_MONO => "Composition-Unsaturation-Monospecific.hover",
            UNSATURATION_STEREO => "Composition-Unsaturation-Stereospecific.hover",
        }
    }
}

impl Display for Composition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MASS_MONO => f.write_str("Mass-Monospecific"),
            MASS_STEREO => f.write_str("Mass-Stereospecific"),
            ECN_MONO => f.write_str("EquivalentCarbonNumber-Monospecific"),
            ECN_STEREO => f.write_str("EquivalentCarbonNumber-Stereospecific"),
            SPECIES_MONO => f.write_str("Species-Monospecific"),
            SPECIES_POSITIONAL => f.write_str("Species-Positionalspecific"),
            SPECIES_STEREO => f.write_str("Species-Stereospecific"),
            TYPE_MONO => f.write_str("Type-Monospecific"),
            TYPE_POSITIONAL => f.write_str("Type-Positionalspecific"),
            TYPE_STEREO => f.write_str("Type-Stereospecific"),
            UNSATURATION_MONO => f.write_str("Unsaturation-Monospecific"),
            UNSATURATION_STEREO => f.write_str("Unsaturation-Stereospecific"),
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
