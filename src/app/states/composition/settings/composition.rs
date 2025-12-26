use self::{
    Composition::*,
    Stereospecificity::{Positional, Stereo},
};
use crate::text::Text;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

pub(crate) const COMPOSITIONS: [Composition; 15] = [
    SPECIES_STEREO,
    SPECIES_POSITIONAL,
    SPECIES_MONO,
    TYPE_STEREO,
    TYPE_POSITIONAL,
    TYPE_MONO,
    MASS_STEREO,
    MASS_POSITIONAL,
    MASS_MONO,
    ECN_STEREO,
    ECN_POSITIONAL,
    ECN_MONO,
    UNSATURATION_STEREO,
    UNSATURATION_POSITIONAL,
    UNSATURATION_MONO,
];

// Mass composition, non-stereospecific
pub(crate) const MASS_MONO: Composition = Mass(None);
// Mass composition, positional-specific
pub(crate) const MASS_POSITIONAL: Composition = Mass(Some(Positional));
// Mass composition, stereospecific
pub(crate) const MASS_STEREO: Composition = Mass(Some(Stereo));

// Equivalent carbon number composition, non-stereospecific
pub(crate) const ECN_MONO: Composition = EquivalentCarbonNumber(None);
// Equivalent carbon number composition, positional-specific
pub(crate) const ECN_POSITIONAL: Composition = EquivalentCarbonNumber(Some(Positional));
// Equivalent carbon number composition, stereospecific
pub(crate) const ECN_STEREO: Composition = EquivalentCarbonNumber(Some(Stereo));

// Species composition, non-stereospecific
pub(crate) const SPECIES_MONO: Composition = Species(None);
// Species composition, non-stereospecific, positional-specific
pub(crate) const SPECIES_POSITIONAL: Composition = Species(Some(Positional));
// Species composition, stereospecific
pub(crate) const SPECIES_STEREO: Composition = Species(Some(Stereo));

// Type composition, non-stereospecific
pub(crate) const TYPE_MONO: Composition = Type(None);
// Type composition, non-stereospecific, positional-specific
pub(crate) const TYPE_POSITIONAL: Composition = Type(Some(Positional));
// Type composition, stereospecific
pub(crate) const TYPE_STEREO: Composition = Type(Some(Stereo));

// Unsaturation composition, non-stereospecific, agregation
pub(crate) const UNSATURATION_MONO: Composition = Unsaturation(None);
// Unsaturation composition, positional-specific
pub(crate) const UNSATURATION_POSITIONAL: Composition = Unsaturation(Some(Positional));
// Unsaturation composition, stereospecific
pub(crate) const UNSATURATION_STEREO: Composition = Unsaturation(Some(Stereo));

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Composition {
    EquivalentCarbonNumber(Option<Stereospecificity>),
    Mass(Option<Stereospecificity>),
    Species(Option<Stereospecificity>),
    Type(Option<Stereospecificity>),
    Unsaturation(Option<Stereospecificity>),
}

impl Composition {
    pub(crate) fn new() -> Self {
        SPECIES_STEREO
    }

    pub(crate) fn stereospecificity(&self) -> Option<Stereospecificity> {
        match *self {
            EquivalentCarbonNumber(stereospecificity) => stereospecificity,
            Mass(stereospecificity) => stereospecificity,
            Species(stereospecificity) => stereospecificity,
            Type(stereospecificity) => stereospecificity,
            Unsaturation(stereospecificity) => stereospecificity,
        }
    }

    pub(crate) fn abbreviation_text(&self) -> &'static str {
        match *self {
            ECN_MONO => "Composition_EquivalentCarbonNumber_Monospecific.abbreviation",
            ECN_POSITIONAL => "Composition_EquivalentCarbonNumber_Positionalspecific.abbreviation",
            ECN_STEREO => "Composition_EquivalentCarbonNumber_Stereospecific.abbreviation",
            MASS_MONO => "Composition_Mass_Monospecific.abbreviation",
            MASS_POSITIONAL => "Composition_Mass_Positionalspecific.abbreviation",
            MASS_STEREO => "Composition_Mass_Stereospecific.abbreviation",
            SPECIES_MONO => "Composition_Species_Monospecific.abbreviation",
            SPECIES_POSITIONAL => "Composition_Species_Positionalspecific.abbreviation",
            SPECIES_STEREO => "Composition_Species_Stereospecific.abbreviation",
            TYPE_MONO => "Composition_Type_Monospecific.abbreviation",
            TYPE_POSITIONAL => "Composition_Type_Positionalspecific.abbreviation",
            TYPE_STEREO => "Composition_Type_Stereospecific.abbreviation",
            UNSATURATION_MONO => "Composition_Unsaturation_Monospecific.abbreviation",
            UNSATURATION_POSITIONAL => "Composition_Unsaturation_Positionalspecific.abbreviation",
            UNSATURATION_STEREO => "Composition_Unsaturation_Stereospecific.abbreviation",
        }
    }
}

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            ECN_MONO => "Composition_EquivalentCarbonNumber_Monospecific",
            ECN_POSITIONAL => "Composition_EquivalentCarbonNumber_Positionalspecific",
            ECN_STEREO => "Composition_EquivalentCarbonNumber_Stereospecific",
            MASS_MONO => "Composition_Mass_Monospecific",
            MASS_POSITIONAL => "Composition_Mass_Positionalspecific",
            MASS_STEREO => "Composition_Mass_Stereospecific",
            SPECIES_MONO => "Composition_Species_Monospecific",
            SPECIES_POSITIONAL => "Composition_Species_Positionalspecific",
            SPECIES_STEREO => "Composition_Species_Stereospecific",
            TYPE_MONO => "Composition_Type_Monospecific",
            TYPE_POSITIONAL => "Composition_Type_Positionalspecific",
            TYPE_STEREO => "Composition_Type_Stereospecific",
            UNSATURATION_MONO => "Composition_Unsaturation_Monospecific",
            UNSATURATION_POSITIONAL => "Composition_Unsaturation_Positionalspecific",
            UNSATURATION_STEREO => "Composition_Unsaturation_Stereospecific",
        }
    }

    fn hover_text(&self) -> &'static str {
        match *self {
            ECN_MONO => "Composition_EquivalentCarbonNumber_Monospecific.hover",
            ECN_POSITIONAL => "Composition_EquivalentCarbonNumber_Positionalspecific.hover",
            ECN_STEREO => "Composition_EquivalentCarbonNumber_Stereospecific.hover",
            MASS_MONO => "Composition_Mass_Monospecific.hover",
            MASS_POSITIONAL => "Composition_Mass_Positionalspecific.hover",
            MASS_STEREO => "Composition_Mass_Stereospecific.hover",
            SPECIES_MONO => "Composition_Species_Monospecific.hover",
            SPECIES_POSITIONAL => "Composition_Species_Positionalspecific.hover",
            SPECIES_STEREO => "Composition_Species_Stereospecific.hover",
            TYPE_MONO => "Composition_Type_Monospecific.hover",
            TYPE_POSITIONAL => "Composition_Type_Positionalspecific.hover",
            TYPE_STEREO => "Composition_Type_Stereospecific.hover",
            UNSATURATION_MONO => "Composition_Unsaturation_Monospecific.hover",
            UNSATURATION_POSITIONAL => "Composition_Unsaturation_Positionalspecific.hover",
            UNSATURATION_STEREO => "Composition_Unsaturation_Stereospecific.hover",
        }
    }
}

impl Display for Composition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ECN_MONO => f.write_str("EquivalentCarbonNumber_Monospecific"),
            ECN_POSITIONAL => f.write_str("EquivalentCarbonNumber_Positionalspecific"),
            ECN_STEREO => f.write_str("EquivalentCarbonNumber_Stereospecific"),
            MASS_MONO => f.write_str("Mass_Monospecific"),
            MASS_POSITIONAL => f.write_str("Mass_Positionalspecific"),
            MASS_STEREO => f.write_str("Mass_Stereospecific"),
            SPECIES_MONO => f.write_str("Species_Monospecific"),
            SPECIES_POSITIONAL => f.write_str("Species_Positionalspecific"),
            SPECIES_STEREO => f.write_str("Species_Stereospecific"),
            TYPE_MONO => f.write_str("Type_Monospecific"),
            TYPE_POSITIONAL => f.write_str("Type_Positionalspecific"),
            TYPE_STEREO => f.write_str("Type_Stereospecific"),
            UNSATURATION_MONO => f.write_str("Unsaturation_Monospecific"),
            UNSATURATION_POSITIONAL => f.write_str("Unsaturation_Positionalspecific"),
            UNSATURATION_STEREO => f.write_str("Unsaturation_Stereospecific"),
        }
    }
}

impl Default for Composition {
    fn default() -> Self {
        Self::new()
    }
}

/// Stereospecificity
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Stereospecificity {
    Stereo,
    Positional,
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
