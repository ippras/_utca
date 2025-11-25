use crate::app::states::calculation::Settings;
#[cfg(feature = "markdown")]
use crate::r#const::markdown::{
    CETANE_NUMBER, COLD_FILTER_PLUGGING_POINT, DEGREE_OF_UNSATURATION, IODINE_VALUE,
    LONG_CHAIN_SATURATED_FACTOR, OXIDATION_STABILITY,
};
use egui::{Grid, InnerResponse, Response, TextWrapMode, Ui, Widget, WidgetText};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::UiExt;
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
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=123"));
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=13"));
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=2"));
                });
            ui.end_row();
            for column in self.data_frame.get_columns() {
                self.property(ui, column)?;
            }
            Ok(())
        })
    }

    fn property(&self, ui: &mut Ui, column: &Column) -> PolarsResult<()> {
        let name = column.name();
        #[allow(unused_variables)]
        let response = ui.label(ui.localize(name));
        #[cfg(feature = "markdown")]
        response.on_hover_ui(|ui| {
            ui.markdown(asset(name));
        });
        for series in column.struct_()?.fields_as_series() {
            let mean_series = series.struct_()?.field_by_name("Mean")?;
            let standard_deviation_series = series.struct_()?.field_by_name("StandardDeviation")?;
            let standard_deviation = standard_deviation_series.str()?.first();
            let text = match mean_series.str()?.first() {
                Some(mean)
                    if self.settings.display_standard_deviation
                        && let Some(standard_deviation) = standard_deviation =>
                {
                    WidgetText::from(format!("{mean} {standard_deviation}"))
                }
                Some(mean) => WidgetText::from(mean.to_string()),
                None => WidgetText::from(""),
            };
            let mut response = ui.label(text);
            if response.hovered() {
                // Standard deviation
                if let Some(text) = standard_deviation {
                    response = response.on_hover_ui(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.heading(ui.localize("StandardDeviation"));
                        ui.label(text);
                    });
                }
                // Sample
                if let Some(text) = series.struct_()?.field_by_name("Sample")?.str()?.first() {
                    response = response.on_hover_ui(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.heading(ui.localize("Sample"));
                        ui.label(text);
                    });
                }
            }
        }
        ui.end_row();
        Ok(())
    }
}

impl Widget for Properties<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

#[cfg(feature = "markdown")]
fn asset(index: &str) -> &str {
    match index {
        "CetaneNumber" => CETANE_NUMBER,
        "ColdFilterPluggingPoint" => COLD_FILTER_PLUGGING_POINT,
        "DegreeOfUnsaturation" => DEGREE_OF_UNSATURATION,
        "IodineValue" => IODINE_VALUE,
        "LongChainSaturatedFactor" => LONG_CHAIN_SATURATED_FACTOR,
        "OxidationStability" => OXIDATION_STABILITY,
        _ => "",
    }
}
