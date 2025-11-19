use std::fmt::{self, Display, Formatter};

use self::{
    Composition::*,
    Stereospecificity::{NonStereospecific, Stereospecific},
};
use crate::text::Text;
use serde::{Deserialize, Serialize};

pub(crate) const COMPOSITIONS: [Composition; 12] = [
    SPECIES_STEREO,
    SPECIES_POSITIONAL,
    SPECIES_MONO,
    TYPE_STEREO,
    TYPE_POSITIONAL,
    TYPE_MONO,
    MASS_STEREO,
    MASS_MONO,
    ECN_STEREO,
    ECN_MONO,
    UNSATURATION_STEREO,
    UNSATURATION_MONO,
];

// Mass composition, non-stereospecific, agregation
pub(crate) const MASS_MONO: Composition = Mass(NonStereospecific(Agregation));
// Mass composition, stereospecific
pub(crate) const MASS_STEREO: Composition = Mass(Stereospecific);

// Equivalent carbon number composition, non-stereospecific, agregation
pub(crate) const ECN_MONO: Composition = EquivalentCarbonNumber(NonStereospecific(Agregation));
// Equivalent carbon number composition, stereospecific
pub(crate) const ECN_STEREO: Composition = EquivalentCarbonNumber(Stereospecific);

// Species composition, non-stereospecific, permutation
pub(crate) const SPECIES_MONO: Composition =
    Species(NonStereospecific(Permutation { positional: false }));
// Species composition, non-stereospecific, permutation, positional
pub(crate) const SPECIES_POSITIONAL: Composition =
    Species(NonStereospecific(Permutation { positional: true }));
// Species composition, stereospecific
pub(crate) const SPECIES_STEREO: Composition = Species(Stereospecific);

// Type composition, non-stereospecific, permutation
pub(crate) const TYPE_MONO: Composition =
    Type(NonStereospecific(Permutation { positional: false }));
// Type composition, non-stereospecific, permutation, positional
pub(crate) const TYPE_POSITIONAL: Composition =
    Type(NonStereospecific(Permutation { positional: true }));
// Type composition, stereospecific
pub(crate) const TYPE_STEREO: Composition = Type(Stereospecific);

// Unsaturation composition, non-stereospecific, agregation
pub(crate) const UNSATURATION_MONO: Composition = Unsaturation(NonStereospecific(Agregation));
// Unsaturation composition, stereospecific
pub(crate) const UNSATURATION_STEREO: Composition = Unsaturation(Stereospecific);

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Composition {
    EquivalentCarbonNumber(Stereospecificity<Agregation>),
    Mass(Stereospecificity<Agregation>),
    Species(Stereospecificity<Permutation>),
    Type(Stereospecificity<Permutation>),
    Unsaturation(Stereospecificity<Agregation>),
}

impl Composition {
    pub(crate) fn new() -> Self {
        SPECIES_STEREO
    }

    pub(crate) fn abbreviation_text(&self) -> &'static str {
        match *self {
            ECN_MONO => "Composition-EquivalentCarbonNumber-Monospecific.abbreviation",
            ECN_STEREO => "Composition-EquivalentCarbonNumber-Stereospecific.abbreviation",
            MASS_MONO => "Composition-Mass-Monospecific.abbreviation",
            MASS_STEREO => "Composition-Mass-Stereospecific.abbreviation",
            SPECIES_MONO => "Composition-Species-Monospecific.abbreviation",
            SPECIES_POSITIONAL => "Composition-Species-Positionalspecific.abbreviation",
            SPECIES_STEREO => "Composition-Species-Stereospecific.abbreviation",
            TYPE_MONO => "Composition-Type-Monospecific.abbreviation",
            TYPE_POSITIONAL => "Composition-Type-Positionalspecific.abbreviation",
            TYPE_STEREO => "Composition-Type-Stereospecific.abbreviation",
            UNSATURATION_MONO => "Composition-Unsaturation-Monospecific.abbreviation",
            UNSATURATION_STEREO => "Composition-Unsaturation-Stereospecific.abbreviation",
        }
    }
}

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            ECN_MONO => "Composition-EquivalentCarbonNumber-Monospecific",
            ECN_STEREO => "Composition-EquivalentCarbonNumber-Stereospecific",
            MASS_MONO => "Composition-Mass-Monospecific",
            MASS_STEREO => "Composition-Mass-Stereospecific",
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
            ECN_MONO => "Composition-EquivalentCarbonNumber-Monospecific.hover",
            ECN_STEREO => "Composition-EquivalentCarbonNumber-Stereospecific.hover",
            MASS_MONO => "Composition-Mass-Monospecific.hover",
            MASS_STEREO => "Composition-Mass-Stereospecific.hover",
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
            ECN_MONO => f.write_str("EquivalentCarbonNumber-Monospecific"),
            ECN_STEREO => f.write_str("EquivalentCarbonNumber-Stereospecific"),
            MASS_MONO => f.write_str("Mass-Monospecific"),
            MASS_STEREO => f.write_str("Mass-Stereospecific"),
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
// pub(crate) enum Numeric {
//     Agregation(Stereospecificity<Agregation>),
//     Permutation(Stereospecificity<Permutation>),
// }

/// Stereospecificity
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Stereospecificity<T> {
    Stereospecific,
    NonStereospecific(T),
}

/// Agregation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Agregation;

/// Permutation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Permutation {
    pub(crate) positional: bool,
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
