pub(super) use self::{
    calculation::{Computed as CalculationComputed, Key as CalculationKey},
    composition::{
        FattyAcidComputed as FattyAcidCompositionComputed, FattyAcidKey as FattyAcidCompositionKey,
        TriacylglycerolComputed as TriacylglycerolCompositionComputed,
        TriacylglycerolKey as TriacylglycerolCompositionKey,
    },
};

pub(super) mod calculation;
pub(super) mod composition;
