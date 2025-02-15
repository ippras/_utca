use crate::special::composition::{
    Composition, MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC,
};

// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn hover_text(&self) -> &'static str;
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
