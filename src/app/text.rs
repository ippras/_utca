use crate::special::composition::{
    Composition, MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC,
};

// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn hover_text(&self) -> &'static str;
}

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            MNC => "mass_nonstereospecific_composition.abbreviation",
            MSC => "mass_stereospecific_composition.abbreviation",
            NNC => "equivalent_carbon_number_nonstereospecific_composition.abbreviation",
            NSC => "equivalent_carbon_number_stereospecific_composition.abbreviation",
            SNC => "species_nonstereospecific_composition.abbreviation",
            SPC => "species_positionalspecific_composition.abbreviation",
            SSC => "species_stereospecific_composition.abbreviation",
            TNC => "type_nonstereospecific_composition.abbreviation",
            TPC => "type_positionalspecific_composition.abbreviation",
            TSC => "type_stereospecific_composition.abbreviation",
            UNC => "unsaturation_nonstereospecific_composition.abbreviation",
            USC => "unsaturation_stereospecific_composition.abbreviation",
        }
    }

    fn hover_text(&self) -> &'static str {
        match *self {
            MNC => "mass_nonstereospecific_composition",
            MSC => "mass_stereospecific_composition",
            NNC => "equivalent_carbon_number_nonstereospecific_composition",
            NSC => "equivalent_carbon_number_stereospecific_composition",
            SNC => "species_nonstereospecific_composition",
            SPC => "species_positionalspecific_composition",
            SSC => "species_stereospecific_composition",
            TNC => "type_nonstereospecific_composition",
            TPC => "type_positionalspecific_composition",
            TSC => "type_stereospecific_composition",
            UNC => "unsaturation_nonstereospecific_composition",
            USC => "unsaturation_stereospecific_composition",
        }
    }
}
