pub(super) use self::{
    calculation::{
        Computed as CalculationComputed, Key as CalculationKey,
        indices::{Computed as CalculationIndicesComputed, Key as CalculationIndicesKey},
    },
    composition::{
        Computed as CompositionComputed, Key as CompositionKey,
        filtered::{Computed as FilteredCompositionComputed, Key as FilteredCompositionKey},
        indices::{Computed as CompositionIndicesComputed, Key as CompositionIndicesKey},
        species::{Computed as CompositionSpeciesComputed, Key as CompositionSpeciesKey},
        unique::{Computed as UniqueCompositionComputed, Key as UniqueCompositionKey},
    },
};

#[derive(Clone, Copy, Debug)]
enum Mode {
    One,
    Many(u64),
}

pub(super) mod calculation;
pub(super) mod composition;
