pub(super) use self::{
    calculation::{
        Computed as CalculationComputed, Key as CalculationKey,
        display::{Computed as CalculationDisplayComputed, Key as CalculationDisplayKey},
        indices::{Computed as CalculationIndicesComputed, Key as CalculationIndicesKey},
    },
    composition::{
        Computed as CompositionComputed, Key as CompositionKey,
        display::{
            Computed as DisplayCompositionComputed, Key as DisplayCompositionKey,
            Kind as DisplayCompositionKind,
        },
        filtered::{Computed as FilteredCompositionComputed, Key as FilteredCompositionKey},
        species::{Computed as CompositionSpeciesComputed, Key as CompositionSpeciesKey},
        unique::{Computed as UniqueCompositionComputed, Key as UniqueCompositionKey},
    },
    configuration::display::{
        Computed as ConfigurationDisplayComputed, Key as ConfigurationDisplayKey,
    },
};

#[derive(Clone, Copy, Debug)]
enum Mode {
    One,
    Many(u64),
}

pub(super) mod calculation;
pub(super) mod composition;
pub(super) mod configuration;
