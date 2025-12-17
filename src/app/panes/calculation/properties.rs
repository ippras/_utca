#[cfg(feature = "markdown")]
use crate::r#const::markdown::{
    CETANE_NUMBER, COLD_FILTER_PLUGGING_POINT, DEGREE_OF_UNSATURATION,
    EICOSAPENTAENOIC_AND_DOCOSAHEXAENOIC, FISH_LIPID_QUALITY, HEALTH_PROMOTING_INDEX,
    HYPOCHOLESTEROLEMIC_TO_HYPERCHOLESTEROLEMIC, INDEX_OF_ATHEROGENICITY, INDEX_OF_THROMBOGENICITY,
    IODINE_VALUE, LINOLEIC_TO_ALPHA_LINOLENIC, LONG_CHAIN_SATURATED_FACTOR, OXIDATION_STABILITY,
    POLYUNSATURATED_6_TO_POLYUNSATURATED_3, POLYUNSATURATED_TO_SATURATED, TRANS,
    UNSATURATION_INDEX,
};
use crate::{
    app::{states::calculation::settings::Settings, widgets::MeanAndStandardDeviation},
    r#const::{EM_DASH, NAME},
};
#[cfg(feature = "markdown")]
use egui::Popup;
use egui::{Grid, InnerResponse, Response, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::prelude::*;
use egui_phosphor::regular::INFO;
use polars::prelude::*;

/// Properties widget
pub(crate) struct Properties<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
}

impl<'a> Properties<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<()>> {
        Grid::new(ui.auto_id_with("Properties")).show(ui, |ui| -> PolarsResult<()> {
            ui.heading(ui.localize("Property?PluralCategory=one"));
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                .on_hover_localized("StereospecificNumber?number=123");
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                .on_hover_localized("StereospecificNumber?number=13");
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                .on_hover_localized("StereospecificNumber?number=2");
            ui.end_row();
            let columns = self.data_frame.get_columns();
            for row in 0..self.data_frame.height() {
                for column in columns {
                    match column.name().as_str() {
                        NAME => {
                            ui.horizontal(|ui| -> PolarsResult<()> {
                                ui.visuals_mut().button_frame = false;
                                #[allow(unused_variables)]
                                let response = ui.button(INFO);
                                let name = self.data_frame[NAME].str()?.get(row).unwrap_or(EM_DASH);
                                ui.label(ui.localize(name));
                                #[cfg(feature = "markdown")]
                                Popup::menu(&response).show(|ui| {
                                    ui.markdown(asset(name));
                                });
                                Ok(())
                            })
                            .inner?;
                        }
                        name => {
                            MeanAndStandardDeviation::new(&self.data_frame, [name], row)
                                .with_standard_deviation(self.settings.standard_deviation)
                                .with_sample(true)
                                .show(ui)?;
                        }
                    }
                }
                ui.end_row();
            }
            Ok(())
        })
    }

    // fn property(&self, ui: &mut Ui, column: &Column) -> PolarsResult<()> {
    //     let name = column.name();
    //     #[allow(unused_variables)]
    //     let response = ui.label(ui.localize(name));
    //     #[cfg(feature = "markdown")]
    //     response.on_hover_ui(|ui| {
    //         ui.markdown(asset(name));
    //     });
    //     for series in column.struct_()?.fields_as_series() {
    //         MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS123], row)
    //             .with_standard_deviation(self.state.settings.standard_deviation)
    //             .with_sample(true)
    //             .show(ui)?;
    //         // let mean_series = series.struct_()?.field_by_name(MEAN)?;
    //         // let standard_deviation_series = series.struct_()?.field_by_name(STANDARD_DEVIATION)?;
    //         // let standard_deviation = standard_deviation_series.str()?.first();
    //         // let text = match mean_series.str()?.first() {
    //         //     Some(mean)
    //         //         if self.settings.standard_deviation
    //         //             && let Some(standard_deviation) = standard_deviation =>
    //         //     {
    //         //         WidgetText::from(format!("{mean} {standard_deviation}"))
    //         //     }
    //         //     Some(mean) => WidgetText::from(mean.to_string()),
    //         //     None => WidgetText::from(""),
    //         // };
    //         // let mut response = ui.label(text);
    //         // if response.hovered() {
    //         //     // Standard deviation
    //         //     if let Some(text) = standard_deviation {
    //         //         response = response.on_hover_ui(|ui| {
    //         //             ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    //         //             ui.heading(ui.localize(STANDARD_DEVIATION));
    //         //             ui.label(text);
    //         //         });
    //         //     }
    //         //     // Sample
    //         //     if let Some(text) = series.struct_()?.field_by_name("Sample")?.str()?.first() {
    //         //         response = response.on_hover_ui(|ui| {
    //         //             ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    //         //             ui.heading(ui.localize("Sample"));
    //         //             ui.label(text);
    //         //         });
    //         //     }
    //         // }
    //     }
    //     ui.end_row();
    //     Ok(())
    // }
}

impl Widget for Properties<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

#[cfg(feature = "markdown")]
fn asset(name: &str) -> &str {
    match name {
        // "Conjugated" => CONJUGATED,
        // "Monounsaturated" => MONOUNSATURATED,
        // "Polyunsaturated" => POLYUNSATURATED,
        // "Saturated" => SATURATED,
        // "Trans" => TRANS,
        // "Unsaturated-3" => UNSATURATED_3,
        // "Unsaturated-6" => UNSATURATED_6,
        // "Unsaturated-9" => UNSATURATED_9,
        // "Unsaturated" => UNSATURATED,
        // "Unsaturated9" => UNSATURATED9,
        "EicosapentaenoicAndDocosahexaenoic" => EICOSAPENTAENOIC_AND_DOCOSAHEXAENOIC,
        "FishLipidQuality" => FISH_LIPID_QUALITY,
        "HealthPromotingIndex" => HEALTH_PROMOTING_INDEX,
        "HypocholesterolemicToHypercholesterolemic" => HYPOCHOLESTEROLEMIC_TO_HYPERCHOLESTEROLEMIC,
        "IndexOfAtherogenicity" => INDEX_OF_ATHEROGENICITY,
        "IndexOfThrombogenicity" => INDEX_OF_THROMBOGENICITY,
        "LinoleicToAlphaLinolenic" => LINOLEIC_TO_ALPHA_LINOLENIC,
        "Polyunsaturated-6ToPolyunsaturated-3" => POLYUNSATURATED_6_TO_POLYUNSATURATED_3,
        "PolyunsaturatedToSaturated" => POLYUNSATURATED_TO_SATURATED,
        "UnsaturationIndex" => UNSATURATION_INDEX,
        "IodineValue" => IODINE_VALUE,
        // Biodiesel properties
        "CetaneNumber" => CETANE_NUMBER,
        "ColdFilterPluggingPoint" => COLD_FILTER_PLUGGING_POINT,
        "DegreeOfUnsaturation" => DEGREE_OF_UNSATURATION,
        "IodineValue" => IODINE_VALUE,
        "LongChainSaturatedFactor" => LONG_CHAIN_SATURATED_FACTOR,
        "OxidationStability" => OXIDATION_STABILITY,
        _ => "",
    }
}
