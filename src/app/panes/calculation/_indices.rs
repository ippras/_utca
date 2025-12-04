use crate::app::states::calculation::Settings;
#[cfg(feature = "markdown")]
use crate::r#const::markdown::{
    EICOSAPENTAENOIC_AND_DOCOSAHEXAENOIC, FISH_LIPID_QUALITY, HEALTH_PROMOTING_INDEX,
    HYPOCHOLESTEROLEMIC_TO_HYPERCHOLESTEROLEMIC, INDEX_OF_ATHEROGENICITY, INDEX_OF_THROMBOGENICITY,
    LINOLEIC_TO_ALPHA_LINOLENIC, POLYUNSATURATED_6_TO_POLYUNSATURATED_3,
    POLYUNSATURATED_TO_SATURATED, TRANS, UNSATURATION_INDEX,
};
use egui::{Grid, InnerResponse, Response, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::UiExt;
use itertools::Itertools as _;
use polars::prelude::*;

/// Indices widget
pub(crate) struct Indices<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
}

impl<'a> Indices<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<()>> {
        Grid::new(ui.auto_id_with("Indices")).show(ui, |ui| -> PolarsResult<()> {
            ui.heading(ui.localize("Index"));
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                .on_hover_localized("StereospecificNumber?number=123");
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                .on_hover_localized("StereospecificNumber?number=13");
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                .on_hover_localized("StereospecificNumber?number=2");
                });
            ui.end_row();
            for index in self.settings.indices.iter_visible() {
                self.index(ui, index)?;
                ui.end_row();
            }
            Ok(())
        })
    }

    fn index(&self, ui: &mut Ui, index: &str) -> PolarsResult<()> {
        #[allow(unused_variables)]
        let response = ui.label(ui.localize(&format!("Indices_{index}")));
        #[cfg(feature = "markdown")]
        response.on_hover_ui(|ui| {
            ui.markdown(asset(index));
        });
        for column in self.data_frame.get_columns() {
            let series = column.struct_()?.field_by_name(index)?;
            if let Some(mean) = series.struct_()?.field_by_name("Mean")?.f64()?.first() {
                ui.horizontal(|ui| -> PolarsResult<()> {
                    let standard_deviation = series
                        .struct_()?
                        .field_by_name("StandardDeviation")?
                        .f64()?
                        .first();
                    let text = if let Some(standard_deviation) = standard_deviation
                        && self.settings.display_standard_deviation
                    {
                        format!("{mean} ±{standard_deviation}")
                    } else {
                        mean.to_string()
                    };
                    let mut response = ui.label(text);
                    if response.hovered() {
                        if let Some(standard_deviation) = standard_deviation {
                            response = response.on_hover_text(format!("±{standard_deviation}"));
                        }
                        if let Some(array) = series
                            .struct_()?
                            .field_by_name("Array")?
                            .array()?
                            .get_as_series(0)
                            && array.len() > 1
                        {
                            let formated = array.f64()?.iter().format_with(", ", |value, f| {
                                if let Some(value) = value {
                                    f(&value)?;
                                }
                                Ok(())
                            });
                            response.on_hover_text(format!("[{formated}]"));
                        }
                    }
                    Ok(())
                })
                .inner?;
            }
        }
        Ok(())
    }
}

impl Widget for Indices<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

#[cfg(feature = "markdown")]
fn asset(index: &str) -> &str {
    match index {
        "EicosapentaenoicAndDocosahexaenoic" => EICOSAPENTAENOIC_AND_DOCOSAHEXAENOIC,
        "FishLipidQuality" => FISH_LIPID_QUALITY,
        "HealthPromotingIndex" => HEALTH_PROMOTING_INDEX,
        "HypocholesterolemicToHypercholesterolemic" => HYPOCHOLESTEROLEMIC_TO_HYPERCHOLESTEROLEMIC,
        "IndexOfAtherogenicity" => INDEX_OF_ATHEROGENICITY,
        "IndexOfThrombogenicity" => INDEX_OF_THROMBOGENICITY,
        "LinoleicToAlphaLinolenic" => LINOLEIC_TO_ALPHA_LINOLENIC,
        "Polyunsaturated-6ToPolyunsaturated-3" => POLYUNSATURATED_6_TO_POLYUNSATURATED_3,
        "PolyunsaturatedToSaturated" => POLYUNSATURATED_TO_SATURATED,
        "Trans" => TRANS,
        "UnsaturationIndex" => UNSATURATION_INDEX,
        _ => "",
    }
}
