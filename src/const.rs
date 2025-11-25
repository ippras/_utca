#[cfg(feature = "markdown")]
#[rustfmt::skip]
pub(crate) mod markdown {
    use crate::asset;

    // Factors
    pub const ENRICHMENT_FACTOR: &str = asset!("/doc/en/Factors/EnrichmentFactor.md");
    pub const SELECTIVITY_FACTOR: &str = asset!("/doc/en/Factors/SelectivityFactor.md");

    // Correlations
    pub const CORRELATIONS: &str = asset!("/doc/en/Correlations/Correlations.md");

    // Indices
    pub const EICOSAPENTAENOIC_AND_DOCOSAHEXAENOIC: &str = asset!("/doc/en/Indices/EicosapentaenoicAndDocosahexaenoic.md");
    pub const FISH_LIPID_QUALITY: &str = asset!("/doc/en/Indices/FishLipidQuality.md");
    pub const HEALTH_PROMOTING_INDEX: &str = asset!("/doc/en/Indices/HealthPromotingIndex.md");
    pub const HYPOCHOLESTEROLEMIC_TO_HYPERCHOLESTEROLEMIC: &str = asset!("/doc/en/Indices/HypocholesterolemicToHypercholesterolemic.md");
    pub const INDEX_OF_ATHEROGENICITY: &str = asset!("/doc/en/Indices/IndexOfAtherogenicity.md");
    pub const INDEX_OF_THROMBOGENICITY: &str = asset!("/doc/en/Indices/IndexOfThrombogenicity.md");
    pub const LINOLEIC_TO_ALPHA_LINOLENIC: &str = asset!("/doc/en/Indices/LinoleicToAlphaLinolenic.md");
    pub const POLYUNSATURATED_6_TO_POLYUNSATURATED_3: &str = asset!("/doc/en/Indices/Polyunsaturated-6ToPolyunsaturated-3.md");
    pub const POLYUNSATURATED_TO_SATURATED: &str = asset!("/doc/en/Indices/PolyunsaturatedToSaturated.md");
    pub const TRANS: &str = asset!("/doc/en/Indices/Trans.md");
    pub const UNSATURATION_INDEX: &str = asset!("/doc/en/Indices/UnsaturationIndex.md");

    // Properties
    pub const CETANE_NUMBER: &str = asset!("/doc/en/Properties/CetaneNumber.md");
    pub const COLD_FILTER_PLUGGING_POINT: &str = asset!("/doc/en/Properties/ColdFilterPluggingPoint.md");
    pub const DEGREE_OF_UNSATURATION: &str = asset!("/doc/en/Properties/DegreeOfUnsaturation.md");
    pub const IODINE_VALUE: &str = asset!("/doc/en/Properties/IodineValue.md");
    pub const LONG_CHAIN_SATURATED_FACTOR: &str = asset!("/doc/en/Properties/LongChainSaturatedFactor.md");
    pub const OXIDATION_STABILITY: &str = asset!("/doc/en/Properties/OxidationStability.md");
}

#[rustfmt::skip]
pub(crate) mod relative_atomic_mass {
    use atom::prelude::isotopes::*;

    pub(crate) const C: f64 = C::Twelve.relative_atomic_mass().value;
    pub(crate) const H: f64 = H::One.relative_atomic_mass().value;
    pub(crate) const LI: f64 = Li::Seven.relative_atomic_mass().value;
    pub(crate) const N: f64 = N::Fourteen.relative_atomic_mass().value;
    pub(crate) const NA: f64 = Na::TwentyThree.relative_atomic_mass().value;
    pub(crate) const O: f64 = O::Sixteen.relative_atomic_mass().value;

    pub(crate) const CH2: f64 = C + 2.0 * H;
    pub(crate) const NH4: f64 = N + 4.0 * H;
}
